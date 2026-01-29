use vrchatapi::apis::{configuration::Configuration, instances_api::GetInstanceByShortNameError};

use crate::api::{VrchatApiMode, VrchatApiState};

// impl VrchatApiState {
//     pub async fn request_safe<T, F: FnOnce(&Configuration) -> T>(&self, f: F) -> T {
//         if self.mode == VrchatApiMode::Ready && self.config.is_some() {
//             //f(self.config.as_ref().unwrap())
//             GetInstanceByShortNameError
//             vrchatapi::apis::instances_api::get_instance_by_short_name(&self.config.as_ref().unwrap(), &instance_id.as_str()).await
//         } else {
//             panic!("API not ready");
//         }
//     }
// }