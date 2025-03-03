use std::{fmt, io::{self, Read, Write}, net::TcpStream, sync::{Arc, Mutex}, thread};

use serde::{de::value::Error, Deserialize, Serialize};
use crate::team_module;
use crate::communication_module::set_tcp_stream;


#[derive(Serialize, Deserialize, Debug)]
struct SubscribePlayer {
    name: String,
    registration_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SubscribePlayerRequest {
    SubscribePlayer: SubscribePlayer,
}

enum SubscribePlayerResult{
     Ok, 
     Err(String) 
}

struct Player {
    id: u32,
    name: String,
}
impl Player {
    fn new(id: u32, name: String) -> Self {
        Player { id, name }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum RelativeDirection {
    Left,
    Right,
    Back,
    Front
}

impl RelativeDirection {
    pub fn to_string(&self) -> &str {
        match self {
            RelativeDirection::Left   => "Left",
            RelativeDirection::Right  => "Right",
            RelativeDirection::Back   => "Back",
            RelativeDirection::Front  => "Front"
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
        Undefined,                   // 00 
        Open,                        // 01
        Wall,                        // 10
        Exit,                        // 1000 
        Unknown(String)              // unknown bits combination
}

impl Cell {
    pub fn from_bits(bits: &str) -> Self {
        match bits {
            "00" | "1111" => Cell::Undefined,
            "01" | "0000" | "0001" | "0101" => Cell::Open,
            "10" | "11" | "0111" | "0011" | "0010" | "0110"| "1010" => Cell::Wall,
            "1000"| "1001" => Cell::Exit,
            _ => Cell::Unknown(bits.to_string())
        }
    }
}
impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cell::Undefined => write!(f, "U"), // Représentation pour Undefined
            Cell::Open => write!(f, "O"),      // Représentation pour Open
            Cell::Wall => write!(f, "W"),      // Représentation pour Wall
            Cell::Exit => write!(f, "E"),      // Représentation pour Exit
            Cell::Unknown(bits) => write!(f, "?{}", bits), // Représentation pour Unknown
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Hint {
    RelativeCompass { angle: f32 },
    GridSize { columns: u32, rows: u32 },
    Secret(u64),
}
#[derive(Serialize, Deserialize, Debug)]
pub enum ActionError {
    CannotPassThroughWall, 
    NoRunningChallenge, 
    SolveChallengeFirst, 
    InvalidChallengeSolution
}
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerPayload {
    ActionError(ActionError),
    RadarView(String),
    Hint(Hint)
}
#[derive(Serialize, Deserialize, Debug)]
struct Action{
    MoveTo : RelativeDirection
}
#[derive(Serialize, Deserialize, Debug)]
struct ActionMessage {
    Action: Action,
}

pub fn ask_user_for_players(num_players: u32) -> Vec<String> {
    let mut players = Vec::new();

    for i in 0..num_players {
        loop {
            print!("Entrez le nom du joueur {} : ", i + 1);
            io::stdout().flush().unwrap();

            let mut name = String::new();
            io::stdin().read_line(&mut name).expect("Erreur lors de la lecture de l'entrée");

            let name = name.trim().to_string();
            if !name.is_empty() {
                players.push(name);
                break;
            } else {
                println!("Le nom du joueur ne peut pas être vide.");
            }
        }
    }

    players
}

pub fn register_team_and_players() -> std::io::Result<()> {
    
    // Demander à l'utilisateur les noms des équipes
    let teams = team_module::ask_user_for_teams();

    // Enregistrer chaque équipe et ses joueurs
    for team_name in teams {
        println!("Enregistrement de l'équipe : {}", team_name);

        // Enregistrer l'équipe et obtenir le token
        if let Ok(Some(token)) = team_module::create_team(team_name.clone()) {
            println!("Token d'inscription pour l'équipe {} : {}", team_name, token);

            // Demander le nombre de joueurs pour cette équipe
            let num_players: u32 = loop {
                print!("Combien de joueurs pour l'équipe {} ? ", team_name);
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Erreur lors de la lecture de l'entrée");

                match input.trim().parse() {
                    Ok(num) if num > 0 => break num,
                    _ => println!("Veuillez entrer un nombre valide supérieur à 0."),
                }
            };

            // Demander les noms des joueurs
            let players = ask_user_for_players(num_players);

            // Inscrire chaque joueur
            for player_name in players {
                println!("Inscription du joueur {} dans l'équipe {}...", player_name, team_name);
                if let Err(err) = subscribe_player( player_name, token.clone()) {
                    println!("Erreur lors de l'inscription du joueur : {}", err);
                }
            }
        } else {
            println!("Échec de l'enregistrement de l'équipe {}.", team_name);
        }
    }

    Ok(())
}




// Fonction pour inscrire un joueur
pub fn subscribe_player(name: String, registration_token: String) -> std::io::Result<()> {

    let stream = set_tcp_stream()?;
    // Créer un thread pour gérer l'inscription du joueur
    let play=thread::spawn(move || {
        if let Err(err) = handle_player(stream, name, registration_token) {
            eprintln!("Erreur lors de l'inscription du joueur : {}", err);
           
        }
    });
    play.join();
    Ok(())

  
}


fn handle_player(mut stream: TcpStream, name: String, registration_token: String) -> io::Result<()> {
    // Envoyer la requête d'inscription
    let request = SubscribePlayerRequest {
        SubscribePlayer: SubscribePlayer {
            name: name.clone(),
            registration_token: registration_token.clone(),
        },
    };

    // Sérialiser la requête en JSON
    let serialized = serde_json::to_string(&request)?;

    // Envoyer la longueur du message (4 bytes)
    let length = serialized.len() as u32;
    stream.write_all(&length.to_le_bytes())?;

    // Envoyer le message JSON
    stream.write_all(serialized.as_bytes())?;

    // Lire la réponse du serveur
    let mut payload_size_buffer = [0u8; 4];
    stream.read_exact(&mut payload_size_buffer).unwrap();
    let payload_size = u32::from_le_bytes(payload_size_buffer) as usize;

    let mut response = vec![0u8; payload_size];
    stream.read_exact(&mut response).unwrap();
    let server_response = serde_json::to_string(&response);
    match server_response {
        Ok(payload)=>{
            println!("Inscription OK ");
        }
        Err(error)=>{
            eprintln!("Erreur d'inscription : {}", error);
            return Err(error.into());
        }
    }
    let mut direction_hint :Option<RelativeDirection> = None;
    // Boucle pour gérer les interactions du joueur
    loop {
        let mut payload_size_buffer = [0u8; 4];
        stream.read_exact(&mut payload_size_buffer).unwrap();
        let payload_size = u32::from_le_bytes(payload_size_buffer) as usize;

        let mut response = vec![0u8; payload_size];
        stream.read_exact(&mut response).unwrap();
        let server_response= match serde_json::from_slice(&response){
           
            Ok(payload) => {
                payload
               
            }
            Err(err) => {
                eprintln!("Erreur lors de la lecture des données de {} : {}", name, err);
                break;
            }
        };
        match server_response {
            ServerPayload::RadarView(view) => {
                println!("Message RadarView reçu de {} : {}", name, view);
                match decoder(&view){
                    Ok(radar_view)=>{
                        let direction = move_player(radar_view, direction_hint);
                        let request = ActionMessage {
                            Action : Action {
                                MoveTo: {direction}
                            }
                            
                        };
                    
                        // Sérialiser la requête en JSON
                        let serialized = serde_json::to_string(&request);
                        match serialized{
                            Ok(json)=>{
                                let length = json.len() as u32;
                                stream.write_all(&length.to_le_bytes())?;
                    
                                // Envoyer le message JSON
                                stream.write_all(json.as_bytes())?;
                    
                            }
                            Err(err)=>{
                                eprintln!("error : {}", err)
                            }
                        }
                    }
                    Err(err)=>{
                       eprintln!("error : {}", err)
                    }
                }

            }
            ServerPayload::Hint(hint) => {
                println!("Message Hint reçu de {} : {:?}", name, hint);
                match hint {
                    Hint::RelativeCompass{ angle } =>{ 
                        direction_hint = Some(direction_from_angle(angle));  
                    
                    }
                    Hint::GridSize {columns, rows } => {

                    }
                    Hint::Secret(secret)=>{

                    }
                    
                }
                // Traiter les données binaires
            }
            ServerPayload::ActionError(error)=>{
                println!("Message ActionError reçu de {} : {:?}", name, error);
            }
        }
    
    }

    Ok(())
}

pub fn base64_decode(encoded: &str) -> Result<Vec<u8>, String> {
    // Table base64
    let base64_table: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
        .chars()
        .collect();

    // Ignorer les espaces
    let encoded = encoded.trim();

    // Calculer le nombre de caractères de padding manquants
    let padding_len = match encoded.len() % 4 {
        0 => 0,      // Aucun padding nécessaire
        2 => 2,      // 2 caractères de padding nécessaires
        3 => 1,      // 1 caractère de padding nécessaire
        _ => return Err("Chaîne base64 invalide : longueur incorrecte".to_string()), // Longueur invalide
    };

    // Convertir chaque caractère en son index dans la table base64
    let indices: Vec<u8> = encoded
        .chars()
        .filter(|&c| c != '=') // Ignorer les caractères de padding
        .map(|c| {
            base64_table
                .iter()
                .position(|&x| x == c)
                .map(|index| index as u8) // Convertir l'index en u8
                .ok_or_else(|| format!("Caractère invalide dans la chaîne base64 : {}", c))
        })
        .collect::<Result<Vec<u8>, String>>()?;

    // Ajouter des indices virtuels pour le padding manquant
    let mut indices_padded = indices.clone();
    for _ in 0..padding_len {
        indices_padded.push(0); // Ajouter des 0 pour simuler le padding
    }

    // Regrouper les indices en blocs de 4 caractères
    let mut decoded = Vec::new();
    for chunk in indices_padded.chunks(4) {
        // Convertir chaque bloc de 4 caractères en 3 octets
        let byte1 = (chunk[0] << 2) | (chunk[1] >> 4);
        let byte2 = ((chunk[1] & 0x0F) << 4) | (chunk[2] >> 2);
        let byte3 = ((chunk[2] & 0x03) << 6) | chunk[3];

        decoded.push(byte1);
        decoded.push(byte2);
        decoded.push(byte3);
    }

    // Retirer les octets de padding si nécessaire
    if padding_len > 0 {
        decoded.truncate(decoded.len() - padding_len);
    }

    Ok(decoded)
}

fn to_binary(bytes: &[u8]) -> Vec<String> {
    bytes
        .iter()
        .map(|byte| format!("{:08b}", byte)) // Convertir chaque octet en binaire sur 8 bits
        .collect()
}

fn to_hex(bytes: &[u8]) -> Vec<String> {
    
    println!("{:?}", bytes);
    bytes
        .iter()
        .map(|byte| format!("{:02x}", byte)) // Convertir chaque octet en hexadécimal sur 2 caractères
        .collect()
}
fn get_first_9_hex_chars_from_array(hex_array: &[String]) -> String {
    // Concaténer toutes les chaînes du tableau
    let concatenated: String = hex_array.concat();
    println!("hexa : {}", concatenated);
   
    // Extraire les 9 premiers caractères
    concatenated[..9].to_string()

}

fn hex_to_binary(hex: &str) -> Vec<Vec<String>>{
    let cells: Vec<String>=hex.chars()
        .map(|c| {
            // Convertir le caractère hexadécimal en valeur numérique
            let value = u8::from_str_radix(&c.to_string(), 16).unwrap();
            // Convertir la valeur numérique en chaîne binaire
            format!("{:04b}", value) // 4 bits par caractère hexadécimal
        })
        .collect();

    let mut radar_cells: Vec<Vec<String>> = vec![];

    for i in 0..3 {
        let row = cells[i * 3..(i + 1) * 3].to_vec(); // Extraire chaque ligne
        radar_cells.push(row);
    }
    radar_cells
    
}
fn decoder(encoded_str:  &str  )->Result<(Vec<Vec<Cell>>), String>{

    match base64_decode(encoded_str) {
        Ok(decoded_bytes) => {
            println!("Octets décodés : {:?}", decoded_bytes);

            // Séparer les octets en deux groupes
            let (first_six_bytes, remaining_bytes) = decoded_bytes.split_at(6);
            
            // Convertir les 6 premiers octets en binaire
            let (first_half, second_half) = split_line_column(&to_binary(first_six_bytes));
            println!("Représentation binaire (6 premiers octets) : {:?} , {:?}", first_half, second_half);

            // Convertir les 5 octets suivants en hexadécimal
            let hex = hex_to_binary(&get_first_9_hex_chars_from_array(&to_hex(remaining_bytes)));
            
            println!("Représentation binaire (5 octets suivants) : {:?}", hex);
            
            let radar_view=create_radar_view(first_half, second_half,hex);
            display_radar_view(&radar_view);
            Ok(radar_view)
        }
        Err(err) => {
            eprintln!("Erreur lors du décodage base64 : {}", err);
            Err(err)
        }
    }
}

fn split_line_column(array: &[String]) -> (Vec<String>, Vec<String>) {
    println!("bits : {:?}", array);
    // Séparer le tableau en deux tranches de 3 éléments
    let mut first_half  = array[..3].to_vec();
    let mut second_half = array[3..].to_vec();
    first_half.reverse();
    second_half.reverse();
    let first_str=first_half.concat();
    let first_tab: Vec<String>=first_str
                    .chars()
                    .collect::<Vec<char>>()
                    .chunks(6)
                    .map(|chunk| chunk.iter().collect::<String>())
                    .collect();

    (first_tab, second_half)
}

fn reader_column_line(array: Vec<String>)->Vec<Vec<Cell>>{
    let mut rows:Vec<Vec<Cell>>= vec![];
    for row in array{
        let mut row_cell : Vec<Cell> = vec![];
        let row_vec: Vec<String>=row
                    .chars()
                    .collect::<Vec<char>>()
                    .chunks(2)
                    .map(|chunk| chunk.iter().collect::<String>())
                    .collect();
        
       
        for element in row_vec{
            row_cell.push(Cell::from_bits(element.as_str()));

        }
        rows.push(row_cell);
    }
    rows
    
}

fn reader_cell(array:Vec<Vec<String>>)->Vec<Vec<Cell>>{
    let mut cells:Vec<Vec<Cell>>= vec![];
    for row in array{
        let mut row_cell:Vec<Cell>= vec![];
        for element in row{
            row_cell.push(Cell::from_bits(element.as_str()));

        }
        cells.push(row_cell);
    }
    cells
}

fn create_radar_view(rows: Vec<String>, columns: Vec<String>, cells: Vec<Vec<String>>)->Vec<Vec<Cell>>{
    let rows_cell= reader_column_line(rows);
    let columns_cell = reader_column_line(columns);
    let radar_cell = reader_cell(cells);
    let mut radar_view: Vec<Vec<Cell>> = vec![vec![Cell::Undefined; 7]; 7];
    let mut j = 0;
    for element in columns_cell.clone(){
        println!("rows {:?}", element)
    }
    for i in (1..radar_view.len()).step_by(2) {
        radar_view[i][0] = columns_cell[j][0].clone();
        radar_view[i][2] = columns_cell[j][1].clone();
        radar_view[i][4] = columns_cell[j][2].clone();
        radar_view[i][6] = columns_cell[j][3].clone();
        j=j+1;
    }
    println!("j {}", j);
    j=0;
    for element in rows_cell.clone(){
        println!("columns {:?}", element)
    }
    for i in (0..radar_view.len()).step_by(2){
       
        radar_view[i][1] = rows_cell[j][0].clone();
        radar_view[i][3] = rows_cell[j][1].clone();
        radar_view[i][5] = rows_cell[j][2].clone();
        j=j+1;
    }
    println!("j {}", j);
    j=0;
    for element in radar_cell.clone(){
        println!("rows {:?}", element)
    }
    for i in (1..radar_view.len()).step_by(2){
        
        radar_view[i][1] = radar_cell[j][0].clone();
        radar_view[i][3] = radar_cell[j][1].clone();
        radar_view[i][5] = radar_cell[j][2].clone();
        j=j+1;
    }
    println!("j {}", j);
    radar_view
}


fn display_radar_view(radar_view: &Vec<Vec<Cell>>) {
    for row in radar_view {
        for cell in row {
            print!("{} ", cell); // Affiche chaque cellule suivie d'un espace
        }
        println!(); // Nouvelle ligne après chaque ligne de la matrice
    }
}

fn move_player(radar_view: Vec<Vec<Cell>>, hint : Option<RelativeDirection>)->RelativeDirection{
    let center = 3;
    if radar_view[center][center] == Cell::Exit {
        println!("Exit reached! Stopping the game.");
        std::process::exit(0);
    }
    let right_cell = radar_view[center][center + 2].clone();
    let front_cell = radar_view[center - 2][center].clone();
    let left_cell  = radar_view[center][center - 2].clone();
    let back_cell = radar_view[center + 2][center].clone();
    let cells = vec![
        (right_cell.clone(), RelativeDirection::Right),
        (front_cell.clone(), RelativeDirection::Front),
        (left_cell.clone(), RelativeDirection::Left),
        (back_cell.clone(), RelativeDirection::Back)
    ];
    match right_cell{
        Cell::Open=>{
            println!("right OK");
        }
        _ =>{}

    }
    match front_cell{
        Cell::Open=>{
            println!("front OK");
        }
        _ =>{}

    }
    match left_cell{
        Cell::Open=>{
            println!("lzft OK");
        }
        _ =>{}

    }
    match back_cell{
        Cell::Open=>{
            println!("back OK");
        }
        _ =>{}

    }
    let open: Vec<RelativeDirection> = cells
        .into_iter()
        .filter(|(cell, _)| *cell == Cell::Open)
        .map(|(_,direction)| direction)
        .collect();

    if let Some(direction) = hint {
        if open.contains(&direction) {
            return direction
        }
    }
    
    let result;
    if open.len()>0{
        result=open[0];
    }else{
        result=RelativeDirection::Right;
    }


    result
    // Envoyer la longueur du message (4 bytes)
   
}

fn direction_from_angle(angle: f32) -> RelativeDirection {
    if angle >= 315.0 || angle < 45.0 {
        RelativeDirection::Front   // North (0°) ➜ Front
    } else if angle >= 45.0 && angle < 135.0 {
        RelativeDirection::Right   // East (90°) ➜ Right
    } else if angle >= 135.0 && angle < 225.0 {
        RelativeDirection::Back    // South (180°) ➜ Back
    } else {
        RelativeDirection::Left    // West (270°) ➜ Left
    }
}