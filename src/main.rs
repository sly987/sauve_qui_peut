use std::{io::{self, Write}, thread};
use team_module::*;
use player_module::*;
use communication_module::*;
mod team_module;

mod player_module;
mod communication_module;


fn main() {
    
    
    loop {
        display_menu();

        // Lire le choix de l'utilisateur
        let mut choice = String::new();
        print!("Votre choix : ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut choice).expect("Erreur lors de la lecture de l'entrée");

        // Gérer le choix avec un match
        match choice.trim() {
            "1" => {
                println!("Enregistrement des équipes...");
                let creation=thread::spawn(move || {
                    if let Err(err) = register_team_and_players() {
                        eprintln!("Erreur lors de l'enregistrement des équipes : {}", err);
                       
                    }
                });
                creation.join();
                break;
            }
            "2" => {
                println!("Au revoir !");
                break;
            }
            _ => {
                println!("Choix invalide. Veuillez entrer 1 ou 2.");
            }
        }
        
    }

    
}
