extern crate json;
use std::sync::Arc;

pub enum MessageBody{
    PlainText{text:String},
    File{bytes:Arc<Vec<u8>>},
    Image{bytes:Arc<Vec<u8>>}
}
pub struct Message{
    messsage_body:MessageBody,
    room_name:String,
    user_id:String
}
impl Message
{
    fn new(user_id:String, room_name:String, msg_body:MessageBody)->Message
    {
        return Message
        {
            user_id:user_id,
            room_name:room_name,
            messsage_body:msg_body
        };
    }
    pub fn get_user_id(&self)->String
    {
        return self.user_id.clone();
    }
    pub fn get_room_name(&self)->String
    {
        return self.room_name.clone();
    }
    pub fn from_str(user_id:String,  json_text: &str)->Result<Message, ()>
    {
        let json_message = match json::parse(json_text)
        {
            Ok(v) => v,
            Err(_) => {
            	println!("{} parsing error!",json_text);
                return Err(());
            }
        };
        // 받은 메시지는 {type:string,hash:string, room:string, value:string}으로 이루어질 것이다. 아니면 잘못 보낸 것임.
        let json_object = match json_message
        {
            json::JsonValue::Object(ref object) => object,
            _ =>
            {
            	println!("line 186 parsing error!");
                return Err(());
            }
        };
        let type_string: String = match json_object.get("type") 
        {
            Some(v) =>match v
            {
                &json::JsonValue::String(ref value)=>value.clone(),
                &json::JsonValue::Short(value) =>value.as_str().to_string(),
                _=>
                {
                    println!("line 196 parsing error!");
                    return Err(());
                }
            },
            None => 
            {
            	println!("line 200 parsing error!");
                return Err(());
            }
        };

        let room: String = match json_object.get("room")
        {
            Some(v) =>match v
            {
                &json::JsonValue::String(ref value)=>value.clone(),
                &json::JsonValue::Short(value) =>value.as_str().to_string(),
                _=>
                {
                    println!("line 211 parsing error!");
                    return Err(());
                }
            },
            None =>
            {
            	println!("line 216 parsing error!");
                return Err(());
            }
        };
		
        let text = match json_object.get("value") {
            Some(v) => match v
            {
                &json::JsonValue::String(ref value)=>value.clone(),
                &json::JsonValue::Short(value) =>value.as_str().to_string(),
                _=>{
                    println!("line 237 parsing error!");
                    return Err(());
                }
            },
            None => 
            {
            	println!("line 242 parsing error!");
                return Err(());
            }
        };
        let chat_type = match type_string.as_str()
        {
            "TEXT" => MessageBody::PlainText { text: text },
            "IMG" => MessageBody::Image { bytes: Arc::new(Vec::new()) },
            "FILE" => MessageBody::File { bytes: Arc::new(Vec::new()) },
			"EXIT" =>
            {
				return Err(());
			}
            _ => {
            	println!("line 244 parsing error!");
                return Err(());
            }
        };
        // TODO:여기서 메시지를 조립하여 반환한다.
        return Ok(Message::new(user_id,room, chat_type));
    }
}