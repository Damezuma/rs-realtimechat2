pub struct User{
    nickname:String,
    hashcode:String,
    entered_room_names:Vec<String>
}
impl User{
    pub fn new(nickname:String, hashcode:String)->User{
        return User{nickname:nickname,hashcode:hashcode,entered_room_names:Vec::new()};
    }
    pub fn get_entered_room_names(&self)->Vec<String>
    {
        return self.entered_room_names.clone();
    }
    pub fn enter_room(&mut self, room_name:String)
    {
        let mut already_entered = false;
        for it in &self.entered_room_names
        {
            if room_name == *it
            {
                already_entered = true;
                break;
            }
        }
        if already_entered == false
        {
            self.entered_room_names.push(room_name);
        }
    }
    pub fn exit_room(&mut self, room_name:String)
    {
        let mut index:Option<usize> = None;
        let len = self.entered_room_names.len();
        for i in 0..len
        {
            if room_name == self.entered_room_names[i]
            {
                index = Some(i);
                break;
            }
        }
        if let Some(index) = index
        {
            self.entered_room_names.swap_remove(index);
        }
    }
    pub fn get_nickname(&self)->String{
        return self.nickname.clone();
    }
    pub fn get_hashcode(&self)->String{
        return self.hashcode.clone();
    }
    
}
