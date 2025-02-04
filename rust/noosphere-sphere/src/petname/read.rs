use anyhow::Result;
use async_trait::async_trait;
use cid::Cid;
use noosphere_core::data::Did;
use noosphere_storage::Storage;
use ucan::crypto::KeyMaterial;

use crate::HasSphereContext;

/// Anything that provides read access to petnames in a sphere should implement
/// [SpherePetnameRead]. A blanket implementation is provided for any container
/// that implements [HasSphereContext].
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait SpherePetnameRead<K, S>
where
    K: KeyMaterial + Clone + 'static,
    S: Storage + 'static,
{
    /// Get the [Did] that is assigned to a petname, if any
    async fn get_petname(&self, name: &str) -> Result<Option<Did>>;

    /// Resolve the petname via its assigned [Did] to a [Cid] that refers to a
    /// point in history of a sphere
    async fn resolve_petname(&self, name: &str) -> Result<Option<Cid>>;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<C, K, S> SpherePetnameRead<K, S> for C
where
    C: HasSphereContext<K, S>,
    K: KeyMaterial + Clone + 'static,
    S: Storage + 'static,
{
    async fn get_petname(&self, name: &str) -> Result<Option<Did>> {
        let sphere = self.to_sphere().await?;
        let identities = sphere.get_address_book().await?.get_identities().await?;
        let address_ipld = identities.get(&name.to_string()).await?;

        Ok(address_ipld.map(|ipld| ipld.did.clone()))
    }

    async fn resolve_petname(&self, name: &str) -> Result<Option<Cid>> {
        let sphere = self.to_sphere().await?;
        let identities = sphere.get_address_book().await?.get_identities().await?;
        let address_ipld = identities.get(&name.to_string()).await?;

        trace!("Recorded address for {name}: {:?}", address_ipld);

        Ok(match address_ipld {
            Some(identity) => {
                let link_record = identity
                    .link_record(self.sphere_context().await?.db())
                    .await;

                match link_record {
                    Some(link_record) => link_record.dereference().await,
                    None => None,
                }
            }
            None => None,
        })
    }
}
