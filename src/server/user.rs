pub struct User{
    nickname:String,
    hashcode:String
}
impl User{
    pub fn new(nickname:String, hashcode:String)->User{
        return User{nickname:nickname,hashcode:hashcode};
    }
    pub fn get_nickname(&self)->String{
        return self.nickname.clone();
    }
    pub fn get_hashcode(&self)->String{
        return self.hashcode.clone();
    }
}
