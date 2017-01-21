extern crate chrono;
extern crate num_cpus;
extern crate crypto_hash;
extern crate json;

use server::ChatServerErr;
use server::user::User;
use server::room::Room;
use server::message::Message;
use std::thread;
use std::time::Duration;
use std::io::{Error, ErrorKind, Read, Write};
use std::io;
use std::collections::BTreeMap;
use std::sync::{Arc,Mutex,Weak};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::net::{TcpListener, TcpStream};
use threadpool::ThreadPool;
use self::chrono::offset::utc::UTC;

enum EventMessage
{
    InitConnectInputPort
    {
        user:User,
        stream:InputStream
    },
    InitConnectOutputPort
    {
        user_hash_code:String,
        stream:TcpStream
    },
    ComeChatMessage
    {
        message:Message
    },
    DisconnectInputSocket
    {
        user_hash_code:String
    },
    DisconnectOutputSocket
    {
        user_hash_code:String
    },
    ChangeNickname
    {
        user_hash_code:String,
        new_nickname:String
    },
    EnterRoom
    {
        message:Message
    },
    ExitRoom
    {
        message:Message
    }

}
fn check_handshake(mut stream: TcpStream)->Result<(User, InputStream), ()>
{
    let mut read_bytes = [0u8; 1024];
    let mut buffer = Vec::<u8>::new();
    let mut message_block_end = false;
    let mut string_memory_block: Vec<u8> = Vec::new();
    stream.set_nodelay(true);
    stream.set_read_timeout(Some(Duration::new(1, 0)));
    stream.set_write_timeout(Some(Duration::new(1, 0)));
    while message_block_end == false
    {
        if let Ok(read_size) = stream.read(&mut read_bytes)
        {
            for i in 0..read_size 
            {
                match (read_bytes[i], message_block_end)
                {
                    (b'\n', false)=>message_block_end = true,
                    ( _ , false)=>string_memory_block.push(read_bytes[i]),
                    _=>buffer.push(read_bytes[i])
                }
            }
        }
        else 
        {
            return Err(());
        }
    }
    // TODO:처음 들어오는 내용은 HandShake헤더다.

    let string_connected_first = String::from_utf8(string_memory_block);
    if let Err(e) = string_connected_first
    {
        println!("{}",e);
        return Err(());
    }
    let string_connected_first = string_connected_first.unwrap();
    
    let json_value = match json::parse(string_connected_first.as_str())
    {
        Ok(v)=>v,
        Err(e)=>
        {
            println!("{}",e);
            return Err(());
        }
    };
    
    let json_value =
    if let json::JsonValue::Object(object) = json_value{
        object
    }
    else
    {
        return Err(());
    };

    let name = 
    if let Some(val) = json_value.get("name")
    {
        match val
        {
            &json::JsonValue::String(ref v)=>v.clone(),
            &json::JsonValue::Short(value) =>value.as_str().to_string(),
            &json::JsonValue::Null=>format!("{}",stream.peer_addr().unwrap()),
            _=>return Err(())
        }
    }
    else
    {
        return Err(());
    };
    let hashing = {
        let value = format!("{}-{}-{}",name,stream.peer_addr().unwrap(),UTC::now());
        let value = value.into_bytes();
        crypto_hash::hex_digest(crypto_hash::Algorithm::SHA512, value)
    };
    let mut return_handshake_json_byte =
    json::stringify(object!
    {
        "status"=>200,
        "id"=>hashing.clone(),
        "name"=>name.clone(),
        "room"=>"lounge"
    }).into_bytes();
    return_handshake_json_byte.push(b'\n');

    if let Err(_) = stream.write_all(&return_handshake_json_byte)
    {
        return Err(());
    }
    let stream = InputStream::new(stream,hashing.clone(),buffer);
    // 핸드셰이크를 완료한 후, 문제가 없으면 유저정보가 들어간 User를 반환한다.
    return Ok((User::new(name,hashing),stream));
    
}
struct InputStream
{
    hashcode:String,
    stream:TcpStream,
    buffer: Vec<u8>
}
impl InputStream
{
    fn new(stream: TcpStream, hashcode:String,buffer:Vec<u8>) -> InputStream
    {
        return InputStream
        {
            hashcode: hashcode,
            stream:stream,
            buffer: buffer,
        }
    }
    fn get_user_id(&self)->String
    {
        return self.hashcode.clone();
    }
    fn read_message(&mut self) -> Result<Message, bool> {
        let mut read_bytes = [0u8; 1024];
        let res_code = self.stream.read(&mut read_bytes);

        if let Err(e) = res_code
        {
        	let exception = match e.kind()
            {
        		ErrorKind::ConnectionRefused |
				ErrorKind::ConnectionReset |
				ErrorKind::ConnectionAborted |
				ErrorKind::NotConnected |
				ErrorKind::Other=>true,
        		_=>false
        	};
            return Err(exception);
        }

        let read_size: usize = res_code.unwrap();
        let mut message_block_end = false;
        let mut string_memory_block: Vec<u8> = Vec::new();
        if self.buffer.len() != 0
        {
            for byte in &self.buffer
            {
                string_memory_block.push(*byte);
            }
            self.buffer.clear();
        }
        for i in 0..read_size
        {
            match (read_bytes[i], message_block_end)
            {
                (b'\n', false)=>message_block_end = true,
                ( _ , false)=>string_memory_block.push(read_bytes[i]),
                _=>self.buffer.push(read_bytes[i])
            }
        }
        if message_block_end == false
        {
            return Err(false);
        }
        let message: String = match String::from_utf8(string_memory_block) 
        {
            Ok(value) => value,
            Err(_) => 
            {
                return Err(true);
            } 
        };
        return match Message::from_str(self.hashcode.clone(), &message)
        {
            Ok(message) => Ok(message),
            Err(_) => Err(true),
        };
    }
}
pub struct Manager
{
    users:Vec<Arc<User>>,
    rooms:BTreeMap<String,Room>,
    input_socket_listener:Arc<TcpListener>,
    output_socket_listener:Arc<TcpListener>,
    file_socket_listener:Arc<TcpListener>,
    input_streams:Vec<Arc< InputStream>>,
    output_streams:BTreeMap< String, Arc< Mutex< TcpStream>>>,
    read_channel_recv:Arc<Mutex<Receiver<Weak<InputStream>>>>,
    read_channel_send:Sender<Weak<InputStream>>
}

impl Manager
{
    pub fn new()->Result<Manager, ChatServerErr>{
        let input_socket_listener = match TcpListener::bind("0.0.0.0:2016") 
        {
            Ok(v) => v,
            Err(_) =>return Err(ChatServerErr::FailedInitializeServer)
        };
        let output_socket_listener = match TcpListener::bind("0.0.0.0:2017")
        {
            Ok(v) => v,
            Err(_) =>return Err(ChatServerErr::FailedInitializeServer)
        };
        let file_socket_listener = match TcpListener::bind("0.0.0.0:2018")
        {
            Ok(v) => v,
            Err(_) =>return Err(ChatServerErr::FailedInitializeServer)
        };
        let (sx,rx) = channel::<Weak<InputStream>>();
        let mut rooms:BTreeMap<String,Room> = BTreeMap::new();
        rooms.insert(String::from("lounge"),Room::new(String::from("lounge")));
        return Ok(Manager
        {
            users:Vec::new(),
            rooms:rooms,
            input_socket_listener:Arc::new(input_socket_listener),
            output_socket_listener:Arc::new(output_socket_listener),
            file_socket_listener:Arc::new(file_socket_listener),
            input_streams:Vec::new(),
            output_streams:BTreeMap::new(),
            read_channel_recv:Arc::new(Mutex::new(rx)),
            read_channel_send:sx
        });
    }
    fn init_accept_input_stream(&self,sender:Sender<EventMessage>,pool:ThreadPool)
    {
        let mut input_listener_rc = self.input_socket_listener.clone();
        thread::spawn(move||
        {
            let input_listener = Arc::get_mut(&mut input_listener_rc);
            if let None = input_listener
            {
                return;
            }
            let input_listener = input_listener.unwrap();
            for stream in input_listener.incoming()
            {
                if let Err(e) = stream
                {
                    println!("{}",e);
                    continue;
                }
                let stream = stream.unwrap();
                let sender = sender.clone();
                //스트림을 받으면 확인을 하자.
                pool.execute(move||
                {
                    let res = check_handshake(stream);
                    if let Err(_) = res
                    {
                        return;
                    }
                    let (user, stream) = res.unwrap();
                    let e = EventMessage::InitConnectInputPort
                    {
                        user:user,
                        stream:stream
                    };
                    sender.send(e);
                });
            }
        });
    }
    
    fn read_user_msg(&self,event_sender:Sender<EventMessage>,pool:ThreadPool)
    {
        let input_stream_recv = self.read_channel_recv.clone();
        let input_stream_send = self.read_channel_send.clone();
        thread::spawn(move||
        {
            let input_stream =match input_stream_recv.lock()
            {
                Ok(receiver)=>receiver.recv(),
                Err(e)=>
                {
                    println!("{}",e);
                    return;
                }
            };
            if let Err(e) = input_stream
            {
                println!("{}",e);
                return;
            }
            let input_stream = input_stream.unwrap();
            let pool = pool.clone();
            let input_stream_send = input_stream_send.clone();
            let event_sender = event_sender.clone();
            pool.execute(move||
            {
                let input_stream = input_stream.upgrade();
                if let None = input_stream
                {
                    return;
                }
                let mut input_stream = input_stream.unwrap();
                let message =match Arc::get_mut(&mut input_stream)
                {
                    Some(ref mut input_stream)=>input_stream.read_message(),
                    None=>return
                };

                if let Err(is_not_timeout) = message
                {
                    if is_not_timeout
                    {
                        //여기에 온 스트림은 타임아웃이 아닌 다른 오류로 여기까지 온 스트림이다. 필요 없으므로 제거한다.
                        //유저도 제거하도록 메시지를 보낸다.
                        let e = EventMessage::DisconnectInputSocket{user_hash_code:input_stream.get_user_id()};
                         event_sender.send(e).unwrap();
                        return;
                    }
                    input_stream_send.send(Arc::downgrade(&input_stream)).unwrap();
                    return;
                }
                input_stream_send.send(Arc::downgrade(&input_stream)).unwrap();
                let message = message.unwrap();
                use server::message::MessageBody;
                let e =
                match message.body
                {
                    MessageBody::PlainText{..}=>Some(EventMessage::ComeChatMessage{message:message}),
                    MessageBody::EnterRoom=>Some(EventMessage::EnterRoom{message:message}),
                    MessageBody::ExitRoom=>Some(EventMessage::ExitRoom{message:message}),
                    _=>
                    {
                        //TODO:그 외에 다른 메시지의 처리도 필요하다.
                        None
                    }
                };
                if let Some(e) = e
                {
                    event_sender.send(e).unwrap();
                }
            });
            
        });
    }
    fn check_outputstream_handshake(mut stream:TcpStream)->Result<(String, TcpStream), ()>
    {
      
        stream.set_read_timeout(Some(Duration::new(60, 0)));
        let mut bytes = [0u8,1024];
        let mut buffer = Vec::<u8>::new();
        'read_loop:loop
        {
            let read_bytes_size = stream.read(&mut bytes);
            if let Err( e ) = read_bytes_size
            {
                println!("{}",e);
                match e.kind()
                {
                    ErrorKind::ConnectionRefused |
                    ErrorKind::ConnectionReset |
                    ErrorKind::ConnectionAborted |
                    ErrorKind::NotConnected |
                    ErrorKind::Other=>return Err(()),
                    _=>continue
                }
            }
            let read_bytes_size:usize = read_bytes_size.unwrap();
            for i in 0..read_bytes_size
            {
                if bytes[i] != b'\n'
                {
                    buffer.push(bytes[i]);
                }
                else
                {
                    break'read_loop;
                }
            }
        }
        let hash = String::from_utf8(buffer);
        if let Err(e) = hash
        {
            println!("{:?}",e);
            return Err(());
        }
        let hash = hash.unwrap();
        return Ok((hash,stream));
    }
    fn init_accept_output_stream(&mut self,event_sender:Sender<EventMessage>,pool:ThreadPool)
    {
        
        let mut listener = self.output_socket_listener.clone();
        //TODO:해야한다.
        thread::spawn(move||
        {
            for stream in listener.incoming()
            {
                if let Err(e) = stream
                {
                    println!("{}",e);
                    continue;
                }
                let mut stream = stream.unwrap();
                let event_sender = event_sender.clone();
                pool.execute(move||
                {
                    let res = Manager::check_outputstream_handshake(stream);
                    if let Err( _ ) = res
                    {
                        return;
                    }
                    let (user_hash_code, stream) = res.unwrap();
                    let e = EventMessage::InitConnectOutputPort{user_hash_code:user_hash_code,stream:stream};
                    event_sender.send(e);
                });
            }
        });
    }
    fn on_init_connect_inputstream(&mut self, user:User, stream:InputStream)
    {
        let s = Arc::new(stream);
        let rc = Arc::new(user);
        self.users.push(rc.clone());
        let mut lounge = self.rooms.get_mut("lounge").unwrap();
        lounge.add_new_user(Arc::downgrade(&rc));
        self.input_streams.push(s.clone());
        self.read_channel_send.send(Arc::downgrade(&s));
    }
    fn on_come_chat_message(&mut self,sender:Sender<EventMessage>, pool:ThreadPool, message:Message)
    {
        //해당 메시지가 보내진 방 안에 있는 유저이 수신하는 소켓를 구한다.
        let mut output_streams:Vec<(String,Arc<Mutex< TcpStream>>)> = Vec::new();
        let room_name = message.get_room_name();
        match self.rooms.get_mut(&room_name)
        {
            None=>{},
            Some(ref mut room)=>
            {
                let users = room.get_users();
                let user_count = users.len();
                for i in 0..user_count
                {
                    let user_wr = &users[user_count];
                    if let Some(user) = Weak::upgrade(&user_wr)
                    {
                        if let Some(v) = self.output_streams.get(&user.get_hashcode())
                        {
                            output_streams.push(
                                (user.get_hashcode(), v.clone())
                            );
                        }
                    }
                    else
                    {
                        room.remove_user(i);
                    }
                }
            }
        }
        //별도의 흐름에서 스레드 큐에 집어 넣는다.
        thread::spawn(move||
        {
            let msg = Arc::new(message.to_send_json_text().unwrap().into_bytes());
            for (user_hash_id, stream) in output_streams
            {
                let msg = msg.clone();
                let sender = sender.clone();
                let lamda = |mut stream:&TcpStream, bytes:Arc<Vec<u8>>|->bool
                {
                    match stream.write_all(&bytes) {
                        Ok(_) => match stream.write_all(b"\n"){
                            Ok(_)=>true,
                            Err(_)=>false
                        },
                        Err(_) => false,
                    }
                };
                pool.execute(move||
                {
                    let  stream = stream.lock();
                    if let Err(e) = stream
                    {
                        println!("{}",e);
                        return;
                    }
                    let mut stream = stream.unwrap();
                    
                    if lamda(&mut stream,msg.clone()) == false
                    {
                        let e = EventMessage::DisconnectOutputSocket{user_hash_code:user_hash_id};
                        sender.send(e).unwrap();
                    }
                });
            }
        });
    }
    fn on_disconnectoutputstream(&mut self, user_hash_id:String)
    {
        //연결이 끊긴 유저 정보를 지운다.
        let len = self.users.len();
        for i in 0..len
        {
            if self.users[i].get_hashcode() == user_hash_id
            {
                self.users.swap_remove(i);
                break;
            }
        }
        self.output_streams.remove(&user_hash_id);

        let len = self.input_streams.len();
        for i in 0..len
        {
            if self.input_streams[i].get_user_id() == user_hash_id
            {
                self.input_streams.swap_remove(i);
                break;
            }
        }
    }
    fn on_disconnectinputstream(&mut self, user_hash_id:String)
    {
        //연결이 끊긴 유저 정보를 지운다.
        let len = self.users.len();
        for i in 0..len
        {
            if self.users[i].get_hashcode() == user_hash_id
            {
                self.users.swap_remove(i);
                break;
            }
        }
        self.output_streams.remove(&user_hash_id);

        let len = self.input_streams.len();
        for i in 0..len
        {
            if self.input_streams[i].get_user_id() == user_hash_id
            {
                self.input_streams.swap_remove(i);
                break;
            }
        }
        //TODO:그리고 해당 유저가 있던 방의 유저들에게 새로운 유저목록을 준다.
    }
    fn on_init_connect_outputstream(&mut self, sender:Sender<EventMessage>, user_hash_code:String, stream:TcpStream)
    {
        for it in &self.users
        {
            if it.get_hashcode() == user_hash_code
            {
                let wrapper = Arc::new(Mutex::new(stream));
                self.output_streams.insert(user_hash_code,wrapper);
                return;
            }
        }
    }
    fn on_change_nickname(&mut self,event_sender:Sender<EventMessage>, user_hash_code:String, new_nickname:String)
    {
        //TODO:이름을 바꾸는 루틴, 나중에 시스템메시지로 변경했다는 알림을 보낸다.
    }
    fn on_enter_room(&mut self,event_sender:Sender<EventMessage>,message:Message)
    {
        //
        let mut user:Option<Weak<User>> = None;
        let user_hash_code = message.get_user_id();
        for it in &self.users
        {
            if it.get_hashcode() == user_hash_code
            {
                user = Some(Arc::downgrade(&it));
                break;
            }
        }
        if let Some(user) = user
        {
            let room_name = message.get_room_name();
            if self.rooms.contains_key(&room_name) == false
            {
                self.rooms.insert(room_name.clone(), Room::new(room_name.clone()));
            }
            let mut room =self.rooms.get_mut(&room_name).unwrap();
            let users_in_room:Vec<Weak<User>> = room.get_users();
            let len = users_in_room.len();
            let mut is_already_in = false;
            for i in 0..len
            {
                if let Some(it) = users_in_room[i].upgrade()
                {
                    if it.get_hashcode() == user_hash_code
                    {
                        is_already_in = true;
                        break;
                    }
                }
            }
            if is_already_in != false
            {
                return;
            }

            //TODO:들어왔다는 시스템 메시지를 보낸다.
            let mut user_rc = user.upgrade().unwrap();
            room.add_new_user(Arc::downgrade(&user_rc.clone()));
            let mut user = Arc::get_mut(&mut user_rc).unwrap();
            user.enter_room(room.get_name());   
        }
    }
    fn on_exit_room(&mut self,event_sender:Sender<EventMessage>,message:Message)
    {

    }
    fn event_procedure(&mut self, pool:ThreadPool,sender:Sender<EventMessage>, receiver:Receiver<EventMessage>)->bool
    {
        loop
        {
            let received_item = receiver.recv();
            if let Err(_) = received_item
            {
                break;
            }
            match received_item.unwrap()
            {
                EventMessage::InitConnectInputPort{user,stream}=>
                    self.on_init_connect_inputstream(user,stream),
                EventMessage::InitConnectOutputPort{user_hash_code,stream}=>
                    self.on_init_connect_outputstream(sender.clone(),user_hash_code,stream),
                EventMessage::ComeChatMessage{message}=>
                    self.on_come_chat_message(sender.clone(),pool.clone(),message),
                EventMessage::DisconnectOutputSocket{user_hash_code}=>
                    self.on_disconnectoutputstream(user_hash_code),
                EventMessage::DisconnectInputSocket{user_hash_code}=>
                    self.on_disconnectinputstream(user_hash_code),
                EventMessage::ChangeNickname{user_hash_code, new_nickname}=>
                    self.on_change_nickname(sender.clone(),user_hash_code,new_nickname),
                EventMessage::EnterRoom{message}=>
                    self.on_enter_room(sender.clone(),message),
                EventMessage::ExitRoom{message}=>
                    self.on_exit_room(sender.clone(),message)
            }
        }
        return false;
    }
   
    pub fn run(&mut self)
    {
        let pool = ThreadPool::new(128);
        let (sender,receiver) = channel::<EventMessage>();
        self.init_accept_input_stream(sender.clone(),pool.clone());
        self.init_accept_output_stream(sender.clone(),pool.clone());
        self.read_user_msg(sender.clone(),pool.clone());
        self.event_procedure(pool.clone(),sender.clone(),receiver);
    }
}