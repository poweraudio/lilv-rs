use crate::node::{Node, Nodes};
use crate::plugin::Plugin;
use crate::Life;
use lilv_sys as lib;
use std::ffi::CStr;
use std::ptr::NonNull;
use std::sync::Arc;

pub struct UI<'a> {
    pub(crate) inner: NonNull<lib::LilvUI>,
    pub(crate) plugin: &'a Plugin,
    pub(crate) life: Arc<Life>,
}

impl<'a> UI<'a> {
    /// # Panics
    /// Panics if it was not possible to get the URI.
    #[must_use]
    pub fn uri(&self) -> Node {
        let _life = self.life.inner.lock();
        let ui = self.inner.as_ptr();

        {
            let ptr = NonNull::new(unsafe { lib::lilv_ui_get_uri(ui) as _ }).unwrap();
            let world = self.plugin.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        }
    }

    #[must_use]
    pub fn classes(&self) -> Option<Nodes> {
        let _life = self.life.inner.lock();
        let ui = self.inner.as_ptr();

        Some({
            let inner = NonNull::new(unsafe { lib::lilv_ui_get_classes(ui) as _ })?;
            let world = self.plugin.life.clone();
            Nodes { inner, life: world }
        })
    }

    #[must_use]
    pub fn is_a(&self, class_uri: &Node) -> bool {
        let _life = self.life.inner.lock();
        let ui = self.inner.as_ptr();
        let class_uri = class_uri.inner.as_ptr();

        unsafe { lib::lilv_ui_is_a(ui, class_uri) }
    }

    #[must_use]
    pub fn is_supported<S>(
        &self,
        container_type: &Node,
        ui_type: Option<&mut Option<Node>>,
    ) -> UISupportQuality
    where
        S: UISupport,
    {
        let ui = self.inner.as_ptr();
        let container_type = container_type.inner.as_ptr();

        let mut ui_type_ptr = std::ptr::null();

        let quality = UISupportQuality(unsafe {
            let _life = self.life.inner.lock();
            lib::lilv_ui_is_supported(
                ui,
                Some(supported_func::<S>),
                container_type,
                ui_type
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |_| &mut ui_type_ptr as _),
            )
        });

        if let Some(ui_type) = ui_type {
            let ptr = match NonNull::new(ui_type_ptr as _) {
                Some(ptr) => ptr,
                None => return UISupportQuality(0),
            };
            *ui_type = Some({
                let world = self.plugin.life.clone();
                Node {
                    inner: ptr,
                    borrowed: true,
                    life: world,
                }
            });
        }

        quality
    }

    #[must_use]
    pub fn bundle_uri(&self) -> Option<Node> {
        let ui = self.inner.as_ptr();
        let _life = self.life.inner.lock();

        Some({
            let ptr = NonNull::new(unsafe { lib::lilv_ui_get_bundle_uri(ui) as _ })?;
            let world = self.plugin.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        })
    }

    /// Get the uri for the binary.
    #[must_use]
    pub fn binary_uri(&self) -> Option<Node> {
        let _life = self.life.inner.lock();
        let ui = self.inner.as_ptr();

        Some({
            let ptr = NonNull::new(unsafe { lib::lilv_ui_get_binary_uri(ui) as _ })?;
            let world = self.plugin.life.clone();
            Node {
                inner: ptr,
                borrowed: true,
                life: world,
            }
        })
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UISupportQuality(pub u32);

pub trait UISupport {
    fn supported(container: &str, ui: &str) -> UISupportQuality;
}

unsafe extern "C" fn supported_func<S: UISupport>(
    container_type_uri: *const i8,
    ui_type_uri: *const i8,
) -> u32 {
    S::supported(
        CStr::from_ptr(container_type_uri).to_str().unwrap(),
        CStr::from_ptr(ui_type_uri).to_str().unwrap(),
    )
    .0
}
