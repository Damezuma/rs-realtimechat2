use std::sync::{Weak,Mutex};
use server::user::User;
use std::collections::BTreeMap;
pub struct Room
{
    name:String,
    users:Mutex<BTreeMap<String, Weak<User>>>
}
impl Room
{
    pub fn new(name:String)->Room
    {
        return Room
        {
            name:name,
            users:Mutex::new(BTreeMap::new())
        };
    }
    pub fn get_name(&self)->String
    {
        return self.name.clone();
    }
    pub fn add_new_user(&self,user_hash_id:String, user:Weak<User>)
    {
        let mut users = self.users.lock().unwrap();
        users.insert(user_hash_id,user);
    }
    pub fn get_users(&self)->BTreeMap<String, Weak<User>>
    {
        let users = self.users.lock().unwrap();
        return users.clone();
    }
    pub fn get_user(&self, user_hash_id:&String)->Option<Weak<User>>
    {
        let users = self.users.lock().unwrap();
        return match users.get(user_hash_id)
        {
            Some(user)=>Some(user.clone()),
            None=>None
        };
    }
    pub fn remove_user(&self, user_hash_id:&String)
    {
        let mut users = self.users.lock().unwrap();
        users.remove(user_hash_id);
    }
}