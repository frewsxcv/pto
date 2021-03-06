use rustc_serialize::json::Json;
use rustc_serialize::json;
use matrix::json as mjson;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct EventID {
    pub id: String,
    pub homeserver: String
}

impl EventID {
    pub fn from_str(s: &str) -> Self {
        let parts: Vec<&str> = s.split(":").collect();
        EventID {
            id: parts[0][1..].to_string(),
            homeserver: parts[1].to_string()
        }
    }
}

#[derive(Clone, Debug)]
pub struct UserID {
    pub nickname: String,
    pub homeserver: String 
}

impl UserID {
    pub fn from_str(s: &str) -> Self {
        let parts: Vec<&str> = s.split(":").collect();
        UserID {
            nickname: parts[0][1..].to_string(),
            homeserver: parts[1].to_string()
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RoomID {
    pub id: String,
    pub homeserver: String
}

impl fmt::Display for RoomID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "!{}:{}", self.id, self.homeserver)
    }
}

impl RoomID {
    pub fn from_str(s: &str) -> Self {
        let parts: Vec<&str> = s.split(":").collect();
        RoomID {
            id: parts[0][1..].to_string(),
            homeserver: parts[1].to_string()
        }
    }
}

#[derive(Debug)]
pub enum MembershipAction {
    Join,
    Leave,
    Ban,
    Invite,
}

impl MembershipAction {
    pub fn from_str(s: &str) -> Self {
        match s {
            "join" => MembershipAction::Join,
            "leave" => MembershipAction::Leave,
            "ban" => MembershipAction::Ban,
            "invite" => MembershipAction::Invite,
            _ => panic!("unknown membership action {:?}", s)
        }
    }
}

#[derive(Debug)]
pub enum RoomEvent {
    CanonicalAlias(String),
    JoinRules(String),
    Membership(UserID, MembershipAction),
    HistoryVisibility(String),
    Create,
    Aliases(Vec<String>),
    Message(UserID, String),
    PowerLevels,
    Name(UserID, String),
    Avatar(UserID, String),
    Topic(UserID, String),
    Unknown(String, Json)
}

#[derive(Debug)]
pub struct TypingEvent {
    pub users: Vec<UserID>,
    pub room: RoomID,
}

#[derive(Debug)]
pub enum EventData {
    Room(RoomID, RoomEvent),
    Typing(TypingEvent),
    Presence(PresenceEvent),
    Unknown(String, Json)
}

impl EventData {
    pub fn type_str(&self) -> String {
        match self {
            &EventData::Room(_, RoomEvent::Message(_, _)) =>
                "m.room.message".to_string(),
            &EventData::Room(_, RoomEvent::CanonicalAlias(_)) =>
                "m.room.canonical_alias".to_string(),
            &EventData::Room(_, RoomEvent::JoinRules(_)) =>
                "m.room.join_rules".to_string(),
            &EventData::Room(_, RoomEvent::Membership(_, _)) =>
                "m.room.member".to_string(),
            &EventData::Room(_, RoomEvent::HistoryVisibility(_)) =>
                "m.room.history_visibility".to_string(),
            &EventData::Room(_, RoomEvent::Create )=>
                "m.room.create".to_string(),
            &EventData::Room(_, RoomEvent::Aliases(_)) =>
                "m.room.aliases".to_string(),
            &EventData::Room(_, RoomEvent::PowerLevels) =>
                "m.room.power_levels".to_string(),
            &EventData::Room(_, RoomEvent::Name(_, _)) =>
                "m.room.name".to_string(),
            &EventData::Room(_, RoomEvent::Avatar(_, _)) =>
                "m.room.avatar".to_string(),
            &EventData::Room(_, RoomEvent::Topic(_, _)) =>
                "m.room.topic".to_string(),
            &EventData::Room(_, RoomEvent::Unknown(ref unknown_type, _)) =>
                format!("m.room.{}", unknown_type),
            &EventData::Typing(_) =>
                "m.typing".to_string(),
            &EventData::Presence(_) =>
                "m.presence".to_string(),
            &EventData::Unknown(ref unknown_type, _) => unknown_type.clone()
        }
    }

    pub fn to_json(&self) -> json::Json {
        let mut ret = json::Object::new();
        match self {
            &EventData::Room(ref _id, ref evt) => {
                match evt {
                    &RoomEvent::Message(_, ref text) => {
                        ret.insert("msgtype".to_string(), json::Json::String("m.text".to_string()));
                        ret.insert("body".to_string(), json::Json::String(text.clone()));
                    },
                    _ => unreachable!()
                }
            },
            _ => unreachable!()
        }
        json::Json::Object(ret)
    }
}

#[derive(Debug)]
pub struct Event {
    pub id: Option<EventID>,
    pub data: EventData
}

#[derive(Debug)]
pub struct PresenceEvent {
    pub presence: String,
    pub user: UserID
}

impl Event {
    pub fn from_json(json: &Json) -> Self {
        let tokens: Vec<&str> = mjson::string(json, "type").trim().split(".").collect();
        let id = match json.as_object().unwrap().get("event_id") {
            Some(i) => Some(EventID::from_str(i.as_string().unwrap())),
            None => None
        };
        if tokens[0] != "m" {
            Event {
                id: id,
                data: EventData::Unknown(json.as_object().unwrap().get("type").unwrap().as_string().unwrap().to_string(), json.clone()),
            }
        } else {
            Event {
                id: id,
                data: match tokens[1] {
                    "room" =>
                        Self::from_room_json(tokens[2], json),
                    "typing" =>
                        EventData::Typing(TypingEvent {
                            users: vec![],
                            room: RoomID::from_str(mjson::string(json, "room_id"))
                        }),
                    "presence" =>
                        EventData::Presence(PresenceEvent{
                            presence: mjson::string(json, "content.presence").to_string(),
                            user: UserID::from_str(mjson::string(json, "content.user_id"))
                        }),
                    e => panic!("Unknown matrix event {:?}!\nRaw JSON: {:?}", e, json)
                }
            }
        }
    }

    fn from_room_json(event_type: &str, json: &Json) -> EventData {
        EventData::Room(
            RoomID::from_str(mjson::string(json, "room_id")),
            match event_type {
                "canonical_alias" =>
                    RoomEvent::CanonicalAlias(mjson::string(json, "content.alias").to_string()),
                "join_rules" => {
                        if json.find_path(&["content", "join_rules"]) == None {
                            RoomEvent::JoinRules(mjson::string(json, "content.join_rule").to_string())
                        } else {
                            RoomEvent::JoinRules(mjson::string(json, "content.join_rules").to_string())
                        }
                    },
                "member" =>
                    RoomEvent::Membership(UserID::from_str(mjson::string(json, "user_id")), MembershipAction::from_str(mjson::string(json, "content.membership"))),
                "history_visibility" =>
                    RoomEvent::HistoryVisibility(mjson::string(json, "content.history_visibility").to_string()),
                "create" =>
                    RoomEvent::Create,
                "aliases" => {
                    let aliases = mjson::array(json, "content.aliases");
                    let mut alias_list: Vec<String> = vec![];
                    for alias in aliases {
                        alias_list.push(alias.as_string().unwrap().to_string());
                    }
                    RoomEvent::Aliases(alias_list)
                },
                "power_levels" =>
                    RoomEvent::PowerLevels,
                "message" =>
                    RoomEvent::Message(UserID::from_str(mjson::string(json, "user_id")), mjson::string(json, "content.body").to_string()),
                "name" =>
                    RoomEvent::Name(UserID::from_str(mjson::string(json, "user_id")), mjson::string(json, "content.name").to_string()),
                "topic" =>
                    RoomEvent::Topic(UserID::from_str(mjson::string(json, "user_id")), mjson::string(json, "content.topic").to_string()),
                "avatar" =>
                    RoomEvent::Avatar(UserID::from_str(mjson::string(json, "user_id")), mjson::string(json, "content.url").to_string()),
                unknown_type => RoomEvent::Unknown(unknown_type.to_string(), json.clone())
            }
        )
    }
}
