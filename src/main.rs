mod model;
mod storage;
mod auth;
mod blockchain;

use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use model::{Resident, Payment};
use storage::Storage;
use std::sync::Mutex;
use crate::blockchain::PaymentTx;

use sha2::{ Digest};
use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{Keypair, Signer, PublicKey, SecretKey};
use rand::rngs::OsRng;
use rand::RngCore;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    let storage = web::Data::new(Mutex::new(Storage::load_from_files()?));
    
    /* let secret = auth::get_jwt_secret();
    println!("🔐 Clave secreta cargada: {}", secret); */

    //let token = auth::generate_token(1, "Andrea".to_string(),"admin".to_string());
    //println!("🔐 Token generado: {}", token);
   
    let token = auth::generate_token(999, "admin".to_string(), "admin".to_string());
    println!("Token admin: {}", token);
    if let Some(claims) = auth::verify_token(&token) {
      println!("✅ Token válido para ID: {}, name: {}", claims.sub, claims.name);
    }

    HttpServer::new(move || {
        App::new()
            .app_data(storage.clone())
            .route("/", web::get().to(|| async { HttpResponse::Ok().body("Hola desde Actix") }))
            .route("/add_resident", web::post().to(add_resident))
            .route("/login", web::post().to(login))
            .route("/charge", web::post().to(charge))
            .route("/report", web::get().to(report))
            .service(get_blockchain)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

//Endpoints HTTP
use actix_web::web::{Json, Data, Query};
use serde::{Deserialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct AddResidentRequest {
    id: u32,
    name: String,
    wallet: String,
}

async fn add_resident(
    user: auth::AuthenticatedUser,
    data: Data<Mutex<Storage>>, 
    body: Json<AddResidentRequest>
) -> impl Responder {

    if user.role != "admin" {
        return HttpResponse::Unauthorized().body("Solo administradores pueden agregar residentes.");
    }

    let mut storage = data.lock().unwrap();

    if storage.residents.contains_key(&body.id) {
        return HttpResponse::BadRequest().body("Residente ya existe.");
    }

    // 🔐 Generar clave privada y codificar en base64
    let mut csprng = OsRng;
    let mut secret_bytes = [0u8; 32];
csprng.fill_bytes(&mut secret_bytes);

let secret = SecretKey::from_bytes(&secret_bytes).unwrap();
let public: PublicKey = (&secret).into(); //Derive public key from the secret key 
let _keypair = Keypair { secret, public };//Keep keypair for potential future use

let private_key_b64 = general_purpose::STANDARD.encode(&secret_bytes);

    let residente = Resident {
        id: body.id,
        name: body.name.clone(),
        wallet: body.wallet.clone(),
        private_key: private_key_b64,
    };

    storage.add_resident(residente);
    storage.save_to_files().unwrap();
    HttpResponse::Ok().body("Residente agregado con nueva clave privada generada.")
}

#[derive(Deserialize)]
struct ChargeRequest {
    id: u32,
    amount: u64,
}

async fn charge(
    user: auth::AuthenticatedUser,
    data: Data<Mutex<Storage>>, 
    body: Json<ChargeRequest>
) -> impl Responder {
    if body.id != user.id {
        return HttpResponse::Unauthorized().body("No puedes registrar pagos para otros residentes.");
    }
    
    let mut storage = data.lock().unwrap();

    if !storage.residents.contains_key(&body.id) {
        return HttpResponse::BadRequest().body("Residente no encontrado.");
    }

    let pago = Payment::new(body.id, body.amount);
    storage.add_payment(pago);
    
    // ⛓️ Creamos una transacción para la blockchain

    

    fn firmar_pago(resident: &Resident, id: u32, amount: u64, timestamp: u64) -> String {
        let msg = format!("{}:{}:{}", id, amount, timestamp);
        let secret_bytes = general_purpose::STANDARD.decode(&resident.private_key).unwrap();
        let secret = SecretKey::from_bytes(&secret_bytes).unwrap();
        let public = PublicKey::from(&secret);
        let keypair = Keypair { secret, public };

        let signature = keypair.sign(msg.as_bytes());
        general_purpose::STANDARD.encode(signature.to_bytes())
    }

    let resident = storage.residents.get(&body.id).unwrap();
    let timestamp = chrono::Utc::now().timestamp() as u64;

    let hash_firma = firmar_pago(resident, body.id, body.amount, timestamp);

    let tx = PaymentTx {
        resident_id: body.id,
        amount: body.amount,
        timestamp,
        hash_firma,
    };

    storage.blockchain.add_block(vec![tx]);
    storage.blockchain.save_to_file("blockchain.json").unwrap();
    storage.save_to_files().unwrap();
    HttpResponse::Ok().body("Pago registrado y añadido a la blockchain.")
}

#[derive(Deserialize)]
struct ReportQuery {
    id: Option<u32>,
}

async fn report(
    user: auth::AuthenticatedUser,
    data: Data<Mutex<Storage>>, 
    query: Query<ReportQuery>
) -> impl Responder {
    let storage = data.lock().unwrap();

    let response = if let Some(id) = query.id {
        if id != user.id || !storage.residents.contains_key(&id) {
            return HttpResponse::BadRequest().body("Residente no encontrado.");
        }

        let saldo = storage.get_resident_balance(id);
        let pagos = storage.list_payments(Some(id));

        serde_json::json!({
            "id": id,
            "saldo": saldo,
            "pagos": pagos
        })
    } else {
        let mut resumen = HashMap::new();

        for res in storage.list_residents() {
            let saldo = storage.get_resident_balance(res.id);
            resumen.insert(res.name.clone(), saldo);
        }

        serde_json::json!({
            "resumen": resumen
        })
    };

    HttpResponse::Ok().json(response)
}

#[derive(Deserialize)]
struct LoginRequest {
    id: u32,
    name: String,
}

async fn login(data: Data<Mutex<Storage>>, body: Json<LoginRequest>) -> impl Responder {
    let storage = data.lock().unwrap();

    if let Some(residente) = storage.residents.get(&body.id) {
        if residente.name == body.name {
            let token = auth::generate_token(residente.id, residente.name.clone(),"user".to_string());
            return HttpResponse::Ok().json(serde_json::json!({ "token": token }));
        }
    }

    HttpResponse::Unauthorized().body("Credenciales inválidas")
}

use actix_web::get;

#[get("/blockchain")]
async fn get_blockchain(data: Data<Mutex<Storage>>) -> impl Responder {
    let storage = data.lock().unwrap();
    HttpResponse::Ok().json(&storage.blockchain.chain)
}