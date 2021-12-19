use tokio::sync::Mutex;

use crate::generated::{
    AuthorRef, Chatroom, GetRoomsRequest, GetRoomsResponse, Message, RoomStatus,
    SendMessageRequest, ZetroContext, ZetroMutations, ZetroQueries, ZetroServerError,
};

/// This will serve as our database
pub struct Db {
    rooms: Vec<Chatroom>,
    /// General purpose ID counter for rooms and messages.
    id_counter: u64,
}

impl Db {
    pub fn new() -> Self {
        let author1 = AuthorRef {
            username: String::from("hal42"),
        };
        let author2 = AuthorRef {
            username: String::from("mitoch0ndria"),
        };
        let author3 = AuthorRef {
            username: String::from("droopydifferential"),
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        // Generate some random seed data.
        Self {
            rooms: vec![
                Chatroom {
                    id: 0,
                    messages: vec![
                        Message {
                            author: author1.clone(),
                            date: now - 4389,
                            id: 192,
                            text: String::from("cats are fun!"),
                        },
                        Message {
                            author: author3.clone(),
                            date: now - 8943,
                            id: 23489,
                            text: String::from(
                                "perhaps, but have you tried solving differential equations?",
                            ),
                        },
                    ],
                    name: String::from("Furry cats"),
                    status: RoomStatus::Active,
                },
                Chatroom {
                    id: 1,
                    messages: vec![
                        Message {
                            author: author2.clone(),
                            date: now - 2903,
                            id: 3489,
                            text: String::from("...so I told them to watch 3b1b..."),
                        },
                        Message {
                            author: author3.clone(),
                            date: now - 328,
                            id: 1290,
                            text: String::from("that is indeed quite entertaining to hear."),
                        },
                        Message {
                            author: author2.clone(),
                            date: now - 328,
                            id: 2390,
                            text: String::from("[mitoch0ndria left the room]"),
                        },
                    ],
                    name: String::from("Differential calculus"),
                    status: RoomStatus::Active,
                },
            ],
            id_counter: 248949,
        }
    }
    /// Gets all the rooms
    pub fn get_rooms(&self) -> &Vec<Chatroom> {
        &self.rooms
    }

    /// Adds a message to room and returns message ID
    pub fn send_message(&mut self, room_id: u64, mut msg: Message) -> u64 {
        let message_id = self.id_counter;
        msg.id = message_id;
        self.id_counter += 1;

        for room in &mut self.rooms {
            if room.id == room_id {
                room.messages.push(msg);
                break;
            }
        }

        message_id
    }
}

pub struct Queries {}
pub struct Mutations {}

#[async_trait::async_trait]
impl ZetroQueries for Queries {
    async fn get_rooms<'a>(
        ctx: &'a ZetroContext,
        request: GetRoomsRequest,
    ) -> Result<GetRoomsResponse, ZetroServerError> {
        let db = ctx.get::<Mutex<Db>>().lock().await;

        let mut rooms = db.get_rooms().clone();
        if let Some(with_status) = request.with_status {
            rooms.retain(|elem| {
                return elem.status == with_status;
            })
        }

        Ok(GetRoomsResponse {
            rooms: rooms.clone(),
        })
    }
}

#[async_trait::async_trait]
impl ZetroMutations for Mutations {
    async fn send_message<'a>(
        ctx: &'a ZetroContext,
        request: SendMessageRequest,
    ) -> Result<u64, ZetroServerError> {
        let mut db = ctx.get::<Mutex<Db>>().lock().await;

        let message_id = db.send_message(request.room_id, request.msg);

        Ok(message_id)
    }
}
