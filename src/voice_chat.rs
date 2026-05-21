use anyhow::{Context, Result};
use ntgcalls::NTgCall;
use std::sync::Arc;

pub struct VoiceChatManager {
    pub ntgcalls: Arc<NTgCall>,
}

impl VoiceChatManager {
    pub fn new(ntgcalls: NTgCall) -> Self {
        Self {
            ntgcalls: Arc::new(ntgcalls),
        }
    }

    pub async fn join_and_play(
        &self,
        chat_id: i64,
        media_url: &str,
        client: &ferogram::Client,
    ) -> Result<()> {
        let call = self.get_input_group_call(chat_id, client).await?;

        let desc = ntgcalls::structures::MediaDescription {
            audio: Some(ntgcalls::structures::AudioDescription {
                input_mode: ntgcalls::structures::InputMode::Shell,
                input: format!(
                    "ffmpeg -re -i \"{}\" -f s16le -ac 2 -ar 48000 -acodec pcm_s16le pipe:1",
                    media_url
                ),
                sample_rate: 48000,
                bits_per_sample: 16,
                channel_count: 2,
            }),
            video: None,
        };

        let offer_params = self.ntgcalls.get_params(chat_id, desc.clone())
            .context("Failed to get params")?;

        let updates = client.invoke(&grammers_tl_types::functions::phone::JoinGroupCall {
            call: grammers_tl_types::enums::InputGroupCall::Call(call),
            join_as: grammers_tl_types::enums::InputPeer::PeerSelf(grammers_tl_types::types::InputPeerSelf {}),
            invite_hash: None,
            params: grammers_tl_types::types::DataJSON { data: offer_params },
            muted: Some(false),
            video_stopped: None,
        }).await.context("Failed to join group call")?;

        let answer_params = Self::extract_answer_params(&updates)
            .context("No answer params in updates")?;

        self.ntgcalls.connect(chat_id, answer_params)
            .context("Failed to connect")?;

        Ok(())
    }

    pub fn change_stream(&self, chat_id: i64, media_url: &str) -> Result<()> {
        let desc = ntgcalls::structures::MediaDescription {
            audio: Some(ntgcalls::structures::AudioDescription {
                input_mode: ntgcalls::structures::InputMode::Shell,
                input: format!(
                    "ffmpeg -re -i \"{}\" -f s16le -ac 2 -ar 48000 -acodec pcm_s16le pipe:1",
                    media_url
                ),
                sample_rate: 48000,
                bits_per_sample: 16,
                channel_count: 2,
            }),
            video: None,
        };
        self.ntgcalls.change_stream(chat_id, desc)
            .context("Failed to change stream")?;
        Ok(())
    }

    pub fn stop(&self, chat_id: i64) -> Result<()> {
        self.ntgcalls.stop(chat_id).context("Failed to stop")?;
        Ok(())
    }

    pub fn pause(&self, chat_id: i64) -> Result<bool> {
        self.ntgcalls.pause(chat_id).context("Failed to pause")
    }

    pub fn resume(&self, chat_id: i64) -> Result<bool> {
        self.ntgcalls.resume(chat_id).context("Failed to resume")
    }

    pub async fn set_volume(&self, chat_id: i64, volume: i32, client: &ferogram::Client) -> Result<()> {
        let call = self.get_input_group_call(chat_id, client).await?;
        client.invoke(&grammers_tl_types::functions::phone::EditGroupCallParticipant {
            call: grammers_tl_types::enums::InputGroupCall::Call(call),
            participant: grammers_tl_types::enums::InputPeer::PeerSelf(grammers_tl_types::types::InputPeerSelf {}),
            muted: None,
            volume: Some(volume),
            raise_hand: None,
            video_stopped: None,
            video_paused: None,
            presentation_paused: None,
        }).await.context("EditGroupCallParticipant failed")?;
        Ok(())
    }

    async fn get_input_group_call(
        &self,
        chat_id: i64,
        client: &ferogram::Client,
    ) -> Result<grammers_tl_types::types::InputGroupCall> {
        let peer = client.resolve_peer(chat_id).await?;
        let full_chat = match peer {
            grammers_tl_types::enums::Peer::Chat(_) => {
                client.invoke(&grammers_tl_types::functions::messages::GetFullChat {
                    chat_id: chat_id,
                }).await?
            }
            grammers_tl_types::enums::Peer::Channel(channel) => {
                let input_channel = grammers_tl_types::types::InputChannel {
                    channel_id: channel.channel_id,
                    access_hash: channel.access_hash,
                };
                client.invoke(&grammers_tl_types::functions::channels::GetFullChannel {
                    channel: grammers_tl_types::enums::InputChannel::Channel(input_channel),
                }).await?
            }
            _ => anyhow::bail!("Unsupported peer type"),
        };

        let call = match full_chat {
            grammers_tl_types::enums::ChatFull::ChatFull(chat_full) => chat_full.call,
            grammers_tl_types::enums::ChatFull::ChannelFull(channel_full) => channel_full.call,
        };

        match call {
            Some(grammers_tl_types::enums::InputGroupCall::Call(call)) => Ok(call),
            Some(grammers_tl_types::enums::InputGroupCall::Slug(_)) => anyhow::bail!("Group call slug not supported"),
            None => anyhow::bail!("No active group call in this chat"),
        }
    }

    fn extract_answer_params(updates: &grammers_tl_types::enums::Updates) -> Result<String> {
        match updates {
            grammers_tl_types::enums::Updates::Updates(u) => {
                for update in &u.updates {
                    if let grammers_tl_types::enums::Update::GroupCallConnection(conn) = update {
                        return Ok(conn.params.data.clone());
                    }
                }
                anyhow::bail!("No GroupCallConnection found")
            }
            grammers_tl_types::enums::Updates::Combined(u) => {
                for update in &u.updates {
                    if let grammers_tl_types::enums::Update::GroupCallConnection(conn) = update {
                        return Ok(conn.params.data.clone());
                    }
                }
                anyhow::bail!("No GroupCallConnection found")
            }
            _ => anyhow::bail!("Unexpected updates type"),
        }
    }
}
