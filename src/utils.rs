use serenity::prelude::*;

use serenity::model::channel::ReactionType;
use serenity::model::id::{ChannelId, UserId};

pub enum Confirmation {
    Yes,
    No,
    InvalidConfirmation,
}

pub async fn ask_confirmation<'a>(
    ctx: &Context,
    author_id: UserId,
    channel_id: ChannelId,
    conf_message: String,
    yes_msg: String,
    no_msg: String,
) -> Result<Confirmation, Box<dyn std::error::Error + Send + Sync>> {
    use std::convert::TryFrom;

    let mut reactions: Vec<ReactionType> = Vec::new();
    reactions.push(ReactionType::try_from("✅").unwrap());
    reactions.push(ReactionType::try_from("❎").unwrap());

    let mut update_sub_msg = channel_id
        .send_message(&ctx.http, |m| {
            m.content(conf_message);
            m.reactions(reactions);
            m
        })
        .await
        .unwrap();

    if let Some(reaction) = &update_sub_msg
        .await_reaction(&ctx)
        .timeout(std::time::Duration::from_secs(10))
        .author_id(author_id)
        .await
    {
        let emoji = &reaction.as_inner_ref().emoji;
        match emoji.as_data().as_str() {
            "✅" => {
                update_sub_msg
                    .edit(ctx, |m| {
                        m.content(yes_msg);
                        m
                    })
                    .await?;
                update_sub_msg.delete_reactions(ctx).await?;
                Ok(Confirmation::Yes)
            }
            "❎" => {
                update_sub_msg
                    .edit(ctx, |m| {
                        m.content(no_msg);
                        m
                    })
                    .await?;
                update_sub_msg.delete_reactions(ctx).await?;
                Ok(Confirmation::No)
            }
            _ => {
                update_sub_msg
                    .reply(ctx, "Please react with ✅ or ❎")
                    .await?;
                Ok(Confirmation::InvalidConfirmation)
            }
        }
    } else {
        update_sub_msg
            .reply(ctx, "No reaction within 10 seconds.")
            .await?;
        Ok(Confirmation::InvalidConfirmation)
    }
}
