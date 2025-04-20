use crate::blockchain::{Blockchain};
use crate::model::{Resident, Payment};
use std::collections::HashMap;
use std::fs::{File};
use std::io::{BufReader, Write};
use serde_json;

pub struct Storage {
    pub residents: HashMap<u32, Resident>,
    pub payments: Vec<Payment>,
    pub blockchain: Blockchain,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            residents: HashMap::new(),
            payments: Vec::new(),
            blockchain: Blockchain::new(),
        }
    }

    pub fn add_resident(&mut self, resident: Resident) {
        self.residents.insert(resident.id, resident);
    }

    pub fn add_payment(&mut self, payment: Payment) {
        self.payments.push(payment);
    }

    pub fn get_resident_balance(&self, resident_id: u32) -> u64 {
        self.payments
            .iter()
            .filter(|p| p.resident_id == resident_id)
            .map(|p| p.amount_copt)
            .sum()
    }

    pub fn list_residents(&self) -> Vec<&Resident> {
        self.residents.values().collect()
    }

    pub fn list_payments(&self, resident_id: Option<u32>) -> Vec<&Payment> {
        self.payments
            .iter()
            .filter(|p| match resident_id {
                Some(id) => p.resident_id == id,
                None => true,
            })
            .collect()
    }

    pub fn save_to_files(&self) -> std::io::Result<()> {
        // Guardar residentes
        let res_json = serde_json::to_string_pretty(&self.residents)?;
        let mut res_file = File::create("residents.json")?;
        res_file.write_all(res_json.as_bytes())?;

        // Guardar pagos
        let pay_json = serde_json::to_string_pretty(&self.payments)?;
        let mut pay_file = File::create("payments.json")?;
        pay_file.write_all(pay_json.as_bytes())?;

         // Guardar blockchain
         self.blockchain.save_to_file("blockchain.json")?;

        Ok(())
    }

    pub fn load_from_files() -> std::io::Result<Self> {
        // Leer residentes
        let res_file = File::open("residents.json").unwrap_or_else(|_| File::create("residents.json").unwrap());
        let reader_res = BufReader::new(res_file);
        let residents: HashMap<u32, Resident> = serde_json::from_reader(reader_res).unwrap_or_else(|_| HashMap::new());

        // Leer pagos
        let pay_file = File::open("payments.json").unwrap_or_else(|_| File::create("payments.json").unwrap());
        let reader_pay = BufReader::new(pay_file);
        let payments: Vec<Payment> = serde_json::from_reader(reader_pay).unwrap_or_else(|_| Vec::new());

        //Inicializa la blockchain  let blockchain = Blockchain::new();
        // Leer blockchain (o crear nueva si no existe)
        let blockchain = Blockchain::load_or_initialize("blockchain.json");

        Ok(Self { residents, payments, blockchain })
    }
}
