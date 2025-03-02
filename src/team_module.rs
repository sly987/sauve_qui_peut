#![allow(unused)]
use std::io::prelude::*;
use std::net::TcpStream;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

#[derive(Serialize, Deserialize)]
struct RegisterTeam {
    name: String,
}

#[derive(Serialize, Deserialize)]
enum Messages {
    RegisterTeam { name: String },
}

#[derive(Serialize, Deserialize, Debug)]
struct RegisterTeamResponse {
    RegisterTeamResult: RegisterTeamResult,
}

#[derive(Serialize, Deserialize, Debug)]
enum RegisterTeamResult {
    Ok(RegistrationSuccess),
    Err(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegistrationSuccess {
    expected_players: u32,
    registration_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AlreadyRegisteredTeam {
    Err: String,
}

pub fn create_team(name: String) -> std::io::Result<()>  {
    // Sérialiser le message en JSON
    let message = Messages::RegisterTeam { name };
    let serialized = serde_json::to_string(&message)?;

    // Connexion au serveur
    let mut stream = TcpStream::connect("localhost:8778")?;

    // Envoyer la longueur du message (4 bytes)
    let length = serialized.len() as u32;
    stream.write_all(&length.to_le_bytes())?;

    // Envoyer le message JSON
    stream.write_all(serialized.as_bytes())?;

    // Lire la longueur de la réponse (4 bytes)
    let mut length_buffer = [0; 4];
    stream.read_exact(&mut length_buffer)?;
    let response_length = u32::from_le_bytes(length_buffer) as usize;

    // Lire la réponse JSON complète
    let mut response_buffer = vec![0; response_length];
    stream.read_exact(&mut response_buffer)?;

    // Convertir en String
    let response_str = String::from_utf8_lossy(&response_buffer);
    println!("Réponse brute: {}", response_str);

    // Désérialiser la réponse
    match serde_json::from_str::<RegisterTeamResponse>(&response_str) {
        Ok(response) => {
            match response.RegisterTeamResult {
                RegisterTeamResult::Ok(success) => {
                    println!("Inscription réussie !");
                    println!("Nombre de joueurs attendus : {}", success.expected_players);
                    println!("Token d'inscription : {}", success.registration_token);
                    return Ok(())
                }
                RegisterTeamResult::Err(error) => {
                    println!("Erreur d'inscription : {}", error);
                  
                }
            }
        }
        Err(err) => {
            println!("Erreur de parsing JSON : {}", err);
         
        }
    }

    Ok(())
}



pub fn ask_user_for_teams() -> Vec<String> {
    let mut teams = Vec::new();

    // Demander le nombre d'équipes
    let num_teams: usize = loop {
        print!("Combien d'équipes souhaitez-vous enregistrer ? ");
        io::stdout().flush().unwrap(); // Forcer l'affichage immédiat

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Erreur lors de la lecture de l'entrée");

        match input.trim().parse() {
            Ok(num) if num > 0 => break num,
            _ => println!("Veuillez entrer un nombre valide supérieur à 0."),
        }
    };

    // Demander les noms des équipes
    for i in 0..num_teams {
        loop {
            print!("Entrez le nom de l'équipe {} : ", i + 1);
            io::stdout().flush().unwrap();

            let mut name = String::new();
            io::stdin().read_line(&mut name).expect("Erreur lors de la lecture de l'entrée");

            let name = name.trim().to_string();
            if !name.is_empty() {
                teams.push(name);
                break;
            } else {
                println!("Le nom de l'équipe ne peut pas être vide.");
            }
        }
    }

    teams
}

pub fn register_team() -> std::io::Result<()> {
    // Demander à l'utilisateur les noms des équipes
    let teams = ask_user_for_teams();

    // Enregistrer chaque équipe
    for team_name in teams {
        println!("Enregistrement de l'équipe : {}", team_name);
        if let Err(err) = create_team(team_name) {
            println!("Erreur lors de l'enregistrement de l'équipe : {}", err);
        }
    }

    Ok(())
}

pub fn display_menu() {
    println!("=== Menu Principal ===");
    println!("1. Enregistrer des équipes");
    println!("2. Quitter");
}