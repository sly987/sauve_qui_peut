use team_module::*;


mod team_module;


fn main() {
    let s: String = "hevllxxor".to_string();
    let start = create_team(s);
    match start {
        Ok(())=>{println!("ok")}
        Err(err)=>{println!("{err}");}
    }

}
// la communication entre l'app et le serveur
// la gestion des joueurs
// score
// challenge