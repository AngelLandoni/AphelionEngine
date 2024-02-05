use shipyard::Component;
use std::ops::Deref;

use crate::scene::asset_server::MeshResourceID;

// TODO(Angel): Find a better name.
#[derive(Component)]
pub struct MeshComponent(pub MeshResourceID);

impl Deref for MeshComponent {
    type Target = MeshResourceID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
