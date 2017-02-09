use std::sync::{Mutex};
struct RecvStream
{
    
}
pub struct User{
    nickname:String,
    hash_id:String,
    entered_room_names:Mutex<Vec<String>>
    send_stream:Mutex<TcpStream>;
    recv_stream:Mutex<TcpStream>;
}
impl User{
    pub fn new(nickname:String, hash_id:String)->User{
        return User{nickname:nickname,hash_id:hash_id,entered_room_names:Mutex::new(Vec::new())};
    }
    pub fn get_entered_room_names(&self)->Vec<String>
    {
        let room_names = self.entered_room_names.lock().unwrap();
        return room_names.clone();
    }
    pub fn recv_message(&self)->Result<Message, bool>
    {

    }
    pub fn send_message(&self)->bool
    {

    }
    pub fn enter_room(&self, room_name:String)
    {
        let mut room_names = self.entered_room_names.lock().unwrap();
        let mut already_entered = false;
        for it in room_names.iter()
        {
            if room_name == *it
            {
                already_entered = true;
                break;
            }
        }
        if already_entered == false
        {
            room_names.push(room_name);
        }
    }
    pub fn exit_room(&self, room_name:&String)
    {
        let mut room_names = self.entered_room_names.lock().unwrap();
        room_names.retain(move|x|
        {
            x != room_name
        });
    }
    pub fn get_nickname(&self)->String{
        return self.nickname.clone();
    }
    pub fn get_hash_id(&self)->String{
        return self.hash_id.clone();
    }
    
}
