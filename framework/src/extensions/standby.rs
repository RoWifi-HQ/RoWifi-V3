use rowifi_models::discord::{
    application::interaction::Interaction,
    id::{marker::MessageMarker, Id},
};
use twilight_gateway::Event;
use twilight_standby::{future::WaitForEventStream, Standby};

pub trait StandbyExtensions {
    fn wait_for_component_interaction(&self, message_id: Id<MessageMarker>) -> WaitForEventStream;
}

impl StandbyExtensions for Standby {
    fn wait_for_component_interaction(&self, message_id: Id<MessageMarker>) -> WaitForEventStream {
        self.wait_for_event_stream(move |event: &Event| {
            if let Event::InteractionCreate(interaction) = &event {
                if let Interaction::MessageComponent(message_component) = &interaction.0 {
                    if message_component.message.id == message_id {
                        return true;
                    }
                }
            }
            false
        })
    }
}
