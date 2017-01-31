extern crate json;
extern crate chrono;

use std::sync::Arc;
use self::chrono::offset::utc::UTC;
pub enum MessageBody{
    PlainText{text:String},
    File{bytes:Arc<Vec<u8>>},
    Image{bytes:Arc<Vec<u8>>},
    EnterRoom,
    ExitRoom,
    ExitServer,
    ChangeName{new_name:String},

}
pub struct Message{
    pub body:MessageBody,
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
            body:msg_body
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
    pub fn to_send_json_text(&self)->Result<String,()>
    {
        let now_time = UTC::now();
        let now_time = now_time.to_rfc2822();
        let json_object = match self.body {
            MessageBody::PlainText {ref text} =>object!
            {
                "type"=>"CHAT_SEND",
                "sender"=>self.get_user_id(),
                "text"=>text.clone(),
                "time"=>now_time,
                "room"=>self.get_room_name()
            },
            _ => {
                object!
                {
                    "sender"=>self.get_user_id(),
                    "text"=>"",
                    "time"=>now_time,
                    "room"=>""
                }
            }
        };
        let send_message = json::stringify(json_object);
        return Ok(send_message);
    }
    pub fn from_str(user_id:String,  json_text: &str)->Result<Message, ()>
    {
        let json_message = match json::parse(json_text)
        {
            Ok(v) => v,
            Err(_) =>
            {
                return Err(());
            }
        };
        // 받은 메시지는 {type:string,hash:string, room:string, value:string}으로 이루어질 것이다. 아니면 잘못 보낸 것임.
        let json_object = match json_message
        {
            json::JsonValue::Object(ref object) => object,
            _ =>
            {
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
                    return Err(());
                }
            },
            None => 
            {
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
                    return Err(());
                }
            },
            None =>
            {
                return Err(());
            }
        };
		
        let text = match json_object.get("value") {
            Some(v) => match v
            {
                &json::JsonValue::String(ref value)=>value.clone(),
                &json::JsonValue::Short(value) =>value.as_str().to_string(),
                _=>{
                    return Err(());
                }
            },
            None => 
            {
                return Err(());
            }
        };
        let chat_type = match type_string.as_str()
        {
            "TEXT" => MessageBody::PlainText { text: text },
            "IMG" => MessageBody::Image { bytes: Arc::new(Vec::new()) },
            "FILE" => MessageBody::File { bytes: Arc::new(Vec::new()) },
			"EXIT" =>MessageBody::ExitServer,
            _ => {
                return Err(());
            }
        };
        // TODO:여기서 메시지를 조립하여 반환한다.
        return Ok(Message::new(user_id,room, chat_type));
    }
}