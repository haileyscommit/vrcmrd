use crate::types::VrcMrdInstanceId;

impl VrcMrdInstanceId {
    pub fn from(full_string: &str) -> VrcMrdInstanceId {
        if (full_string.matches(':').count() != 1) || (!full_string.contains("wrld_")) {
            eprintln!("WARNING: Invalid instance ID format: {}\n(Yell at me to update this if this looks like a single UUID that begins with inst_)", full_string);
            return VrcMrdInstanceId {
                raw: full_string.to_string(),
                world: "".to_string(),
                id: "".to_string(),
                instance_type: "unknown".to_string(),
                owner: None,
                public: false,
                region: "".to_string(),
            };
        }
        let mut parts = full_string.split('~');
        let (world_id, instance_id) = {
            parts
                .nth(0)
                .unwrap_or_default()
                .split_once(':')
                .unwrap_or(("", ""))
        };
        let mut instance_type = "unknown".to_string();
        let mut owner: Option<String> = None;
        let mut region = "".to_string();

        for part in parts.skip(1) {
            if part.starts_with("public(") {
                instance_type = "public".to_string();
                continue;
            }
            if part.starts_with("hidden(") && instance_type == "unknown" {
                instance_type = "friends+".to_string();
                let start = part.find('(').unwrap_or(0) + 1;
                let end = part.find(')').unwrap_or(part.len());
                owner = Some(part[start..end].to_string());
                continue;
            }
            if part.starts_with("friends(") && instance_type == "unknown" {
                instance_type = "friends".to_string();
                let start = part.find('(').unwrap_or(0) + 1;
                let end = part.find(')').unwrap_or(part.len());
                owner = Some(part[start..end].to_string());
                continue;
            }
            if part.starts_with("group(") && instance_type == "unknown" && owner.is_none() {
                //instance_type = "invite+".to_string();
                let start = part.find('(').unwrap_or(0) + 1;
                let end = part.find(')').unwrap_or(part.len());
                owner = Some(part[start..end].to_string());
                continue;
            }
            if part.starts_with("private(") && instance_type == "unknown" {
                instance_type = "invite".to_string();
                let start = part.find('(').unwrap_or(0) + 1;
                let end = part.find(')').unwrap_or(part.len());
                owner = Some(part[start..end].to_string());
                continue;
            }
            if part.starts_with("canRequestInvite")
                && matches!(instance_type.as_str(), "invite" | "unknown")
            {
                instance_type = "invite+".to_string();
                continue;
            }
            if part.starts_with("groupAccessType(") && instance_type == "unknown" {
                let start = part.find('(').unwrap_or(0) + 1;
                let end = part.find(')').unwrap_or(part.len());
                let access_type = &part[start..end];
                instance_type = match access_type {
                    "members" => "group".to_string(),
                    "public" => "groupPublic".to_string(),
                    "plus" => "group+".to_string(),
                    _ => instance_type.clone(),
                };
                continue;
            }
            if part.starts_with("region") {
                let start = part.find('(').unwrap_or(0) + 1;
                let end = part.find(')').unwrap_or(part.len());
                region = part[start..end].to_string();
                continue;
            }
        }
        VrcMrdInstanceId {
            raw: full_string.to_string(),
            world: world_id.to_string(),
            id: instance_id.to_string(),
            public: matches!(instance_type.as_str(), "public" | "group public"),
            instance_type: instance_type,
            owner,
            region,
        }
    }
}
