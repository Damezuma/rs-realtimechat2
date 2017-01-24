use std::sync::Weak;
use server::user::User;
pub struct Room
{
    name:String,
    users:Vec<Weak<User>>
}
impl Room
{
    pub fn new(name:String)->Room
    {
        return Room
        {
            name:name,
            users:Vec::new()
        };
    }
    pub fn get_name(&self)->String
    {
        return self.name.clone();
    }
    pub fn add_new_user(&mut self, user:Weak<User>)
    {
        self.users.push(user);
    }
    pub fn get_users(&self)->Vec<Weak<User>>
    {
        return self.users.clone();
    }
    pub fn remove_user(&mut self, index:usize)
    {
       self.users.swap_remove(index);
    }
    pub fn check_exit_users(&mut self)
    {
        self.users.retain(|ref x|
        {
            match x.upgrade()
            {
                Some(..)=>true,
                None=>false
            }
        });
    }
    pub fn set_users(&mut self, users:Vec<Weak<User>>)
    {
        self.users = users
    }
}