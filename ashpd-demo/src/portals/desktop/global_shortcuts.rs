use std::sync::Arc;

use adw::subclass::prelude::*;
use ashpd::{
    desktop::{
        global_shortcuts::{GlobalShortcuts, NewShortcut},
        Session,
    },
    WindowIdentifier,
};
use gtk::{glib, prelude::*};
use futures_util::lock::Mutex;
use crate::widgets::{PortalPage, PortalPageImpl};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/global_shortcuts.ui")]
    pub struct GlobalShortcutsPage {
        #[template_child]
        pub shortcuts: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub response_group: TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub idle_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub logout_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub user_switch_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub suspend_check: TemplateChild<gtk::CheckButton>,
        pub session: Arc<Mutex<Option<Session<'static>>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GlobalShortcutsPage {
        const NAME: &'static str = "GlobalShortcutsPage";
        type Type = super::GlobalShortcutsPage;
        type ParentType = PortalPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action_async("global_shortcuts.start_session", None, |page, _, _| async move {
                if let Err(err) = page.start_session().await {
                    tracing::error!("Failed to request {}", err);
                }
            });
            klass.install_action_async("global_shortcuts.stop", None, |page, _, _| async move {
                page.stop().await;
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for GlobalShortcutsPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().action_set_enabled("global_shortcuts.stop", false);
        }
    }
    impl WidgetImpl for GlobalShortcutsPage {}
    impl BinImpl for GlobalShortcutsPage {}
    impl PortalPageImpl for GlobalShortcutsPage {}
}

glib::wrapper! {
    pub struct GlobalShortcutsPage(ObjectSubclass<imp::GlobalShortcutsPage>)
        @extends gtk::Widget, adw::Bin;
}

impl GlobalShortcutsPage {
    async fn start_session(&self) -> ashpd::Result<()> {
        let root = self.native().unwrap();
        let imp = self.imp();
        let identifier = WindowIdentifier::from_native(&root).await;
        let shortcuts = imp.shortcuts.text();
        let shortcuts: Vec<_> = shortcuts.as_str().split(' ')
            .map(|desc| {
                NewShortcut::new(desc, "")
            }).collect();

        let global_shortcuts = GlobalShortcuts::new().await?;
        println!("New");
        let session = global_shortcuts.create_session().await?;
        println!("created");
        global_shortcuts.bind_shortcuts(&session, &shortcuts[..], &identifier).await?;
        println!("bound");
        imp.session.lock().await.replace(session);
        self.action_set_enabled("global_shortcuts.stop", true);
        self.action_set_enabled("global_shortcuts.start_session", false);
/*
        let mut state = proxy.receive_state_changed().await?;
        match state
            .next()
            .await
            .expect("Stream exhausted")
            .session_state()
        {
            SessionState::Running => tracing::info!("Session running"),
            SessionState::QueryEnd => {
                tracing::info!("Session: query end");
                proxy.inhibit(&identifier, flags, &shortcuts).await?;
                if let Some(session) = imp.session.lock().await.as_ref() {
                    proxy.query_end_response(session).await?;
                }
            }
            SessionState::Ending => {
                tracing::info!("Ending the session");
            }
        }*/
        Ok(())
    }

    async fn stop(&self) {
        let imp = self.imp();
        self.action_set_enabled("global_shortcuts.stop", false);
        self.action_set_enabled("global_shortcuts.start_session", true);
        if let Some(session) = imp.session.lock().await.take() {
            let _ = session.close().await;
        }
    }
}
