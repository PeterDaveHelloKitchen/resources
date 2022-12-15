
use adw::{prelude::*, subclass::prelude::*};
use anyhow::Context;
use gtk::glib::{self, clone};

use crate::config::PROFILE;
use crate::ui::widgets::progress_box::ResProgressBox;
use crate::utils::memory::{get_available_memory, get_free_swap, get_total_memory, get_total_swap};
use crate::utils::units::{to_largest_unit, Base};
use crate::utils::NaNDefault;

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/me/nalux/Resources/ui/pages/memory.ui")]
    pub struct ResMemory {
        #[template_child]
        pub memory: TemplateChild<ResProgressBox>,
        #[template_child]
        pub swap: TemplateChild<ResProgressBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ResMemory {
        const NAME: &'static str = "ResMemory";
        type Type = super::ResMemory;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ResMemory {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.instance();

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }
        }
    }

    impl WidgetImpl for ResMemory {}
    impl BinImpl for ResMemory {}
}

glib::wrapper! {
    pub struct ResMemory(ObjectSubclass<imp::ResMemory>)
        @extends gtk::Widget, adw::Bin;
}

impl ResMemory {
    pub fn new() -> Self {
        glib::Object::new::<Self>(&[])
    }

    pub fn init(&self) {
        self.setup_signals();
    }

    pub fn setup_signals(&self) {
        let mem_usage_update = clone!(@strong self as this => move || {
            let imp = this.imp();
            let total_mem = get_total_memory().with_context(|| "unable to get total memory").unwrap_or_default();
            let available_mem = get_available_memory().with_context(|| "unable to get available memory").unwrap_or_default();
            let total_swap = get_total_swap().with_context(|| "unable to get total swap").unwrap_or_default();
            let free_swap = get_free_swap().with_context(|| "unable to get free swap").unwrap_or_default();

            let total_mem_unit = to_largest_unit(total_mem as f64, &Base::Decimal);
            let used_mem_unit = to_largest_unit((total_mem - available_mem) as f64, &Base::Decimal);
            let total_swap_unit = to_largest_unit(total_swap as f64, &Base::Decimal);
            let used_swap_unit = to_largest_unit((total_swap - free_swap) as f64, &Base::Decimal);

            imp.memory.set_fraction(1.0 - (available_mem as f64 / total_mem as f64));
            imp.memory.set_percentage_label(&format!("{:.2} {}B / {:.2} {}B", used_mem_unit.0, used_mem_unit.1, total_mem_unit.0, total_mem_unit.1));
            if total_swap == 0 {
                imp.swap.set_progressbar_visible(false);
                imp.swap.set_percentage_label(&gettextrs::gettext("N/A"));
            } else {
                imp.swap.set_fraction(1.0 - (free_swap as f64 / total_swap as f64).nan_default(1.0));
                imp.swap.set_progressbar_visible(true);
                imp.swap.set_percentage_label(&format!("{:.2} {}B / {:.2} {}B", used_swap_unit.0, used_swap_unit.1, total_swap_unit.0, total_swap_unit.1));
            }

            glib::Continue(true)
        });

        glib::timeout_add_seconds_local(1, mem_usage_update);
    }
}
