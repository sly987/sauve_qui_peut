use communication_module::connect_to_server;

mod communication_module;


fn main() {
    let OK = connect_to_server();
    match OK {
        Ok(())=>{println!("ok")}
        Err(err)=>{println!("{err}");}
    }

}
// la communication entre l'app et le serveur
// la gestion des joueurs
// score
// challenge