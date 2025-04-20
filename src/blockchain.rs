use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::fs::{File};
use std::io::{Read, Write};
use std::io::BufReader;
// Define una estructura para representar una transacción de pago.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaymentTx {
    pub resident_id: u32, // Identificador del residente que realiza el pago.
    pub amount: u64,      // Cantidad del pago.
    pub timestamp: u64,   // Marca de tiempo de cuándo se realizó la transacción.
    pub hash_firma: String,
}

// Define una estructura para representar un bloque en la cadena de bloques.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,        // La posición del bloque en la cadena. El primer bloque (Génesis) suele ser 0.
    pub timestamp: u64,    // Marca de tiempo de cuándo se creó el bloque.
    pub transactions: Vec<PaymentTx>, // Una lista de transacciones incluidas en este bloque.
    pub previous_hash: String, // El hash del bloque anterior en la cadena. Esto asegura la integridad de la cadena.
    pub hash: String,         // El hash único de este bloque, calculado a partir de su contenido.
}

// Implementa métodos para la estructura `Block`.
impl Block {
    // Calcula el hash del bloque basándose en su contenido.
    pub fn calculate_hash(&self) -> String {
        // Serializa los datos importantes del bloque (índice, timestamp, transacciones y hash previo) a una cadena JSON.
        let block_data = serde_json::to_string(&(
            self.index,
            self.timestamp,
            &self.transactions,
            &self.previous_hash
        )).unwrap();

        // Crea una nueva instancia del algoritmo de hash Sha256.
        let mut hasher = Sha256::new();
        // Actualiza el hasher con los datos del bloque serializados.
        hasher.update(block_data);
        // Finaliza el hash y lo formatea como una cadena hexadecimal.
        format!("{:x}", hasher.finalize())
    }

    // Crea una nueva instancia de un `Block`.
    pub fn new(index: u64, transactions: Vec<PaymentTx>, previous_hash: String) -> Self {
        // Obtiene la marca de tiempo actual en segundos desde la época (Unix timestamp).
        let timestamp = chrono::Utc::now().timestamp() as u64;
        // Crea una nueva instancia de `Block` con los datos proporcionados y un hash vacío como marcador de posición.
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(), // Placeholder
        };
        // Calcula el hash del bloque recién creado y lo asigna al campo `hash`.
        block.hash = block.calculate_hash();
        // Devuelve el bloque creado.
        block
    }
}

// Define una estructura para representar la cadena de bloques.
#[derive(Serialize, Deserialize, Debug)]
pub struct Blockchain {
    pub chain: Vec<Block>, // Un vector que contiene todos los bloques en la cadena.
}

// Implementa métodos para la estructura `Blockchain`.
impl Blockchain {
    // Crea una nueva instancia de `Blockchain` con el bloque Génesis.
    pub fn new() -> Self {
        // Crea el primer bloque de la cadena (bloque Génesis) con índice 0, sin transacciones y un hash previo de "0".
        let genesis_block = Block::new(0, vec![], "0".to_string());
        // Devuelve una nueva instancia de `Blockchain` con el bloque Génesis como el primer elemento de la cadena.
        Blockchain {
            chain: vec![genesis_block],
        }
    }

    // Añade un nuevo bloque a la cadena de bloques.
    pub fn add_block(&mut self, transactions: Vec<PaymentTx>) {
        // Obtiene el último bloque de la cadena. `unwrap()` se usa aquí asumiendo que la cadena siempre tendrá al menos el bloque Génesis.
        let latest = self.chain.last().unwrap();
        // Crea un nuevo bloque con el índice siguiente al del último bloque, las transacciones proporcionadas y el hash del último bloque como hash previo.
        let block = Block::new(
            latest.index + 1,
            transactions,
            latest.hash.clone(), // Clona el hash del bloque anterior para evitar problemas de propiedad.
        );
        // Añade el nuevo bloque a la cadena.
        self.chain.push(block);
    }

    /// Guarda la cadena de bloques en un archivo JSON.
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

     /// Carga una cadena de bloques desde un archivo JSON.
     pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let blockchain: Blockchain = serde_json::from_str(&contents)?;
        Ok(blockchain)
    }

    /// Crea una nueva instancia de `Blockchain`, cargándola de un archivo si existe.
    pub fn load_or_initialize(path: &str) -> Self {
        let file = File::open(path);
        match file {
            Ok(f) =>{
                let reader = BufReader::new(f);
                serde_json::from_reader(reader).unwrap_or_else(|_| Blockchain::new())
            },
            Err(_) => Blockchain::new(),
            }
    }
        
}