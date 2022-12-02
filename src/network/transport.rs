use std::{sync::Arc, time::Duration};

use libp2p::{
    bandwidth::{self, BandwidthSinks},
    core::{self, muxing::StreamMuxerBox, transport::Boxed},
    identity::Keypair,
    mplex, noise, tcp, websocket, yamux, PeerId, Transport,
};

pub fn new_transport(local_key: Keypair) -> (Boxed<(PeerId, StreamMuxerBox)>, Arc<BandwidthSinks>) {
    let transport = {
        let tcp_config = tcp::Config::new().nodelay(true);
        let desktop_trans: tcp::Transport<tcp::async_io::Tcp> =
            tcp::Transport::new(tcp_config.clone());
        let desktop_trans =
            websocket::WsConfig::new(desktop_trans)
                .or_transport(
                    tcp::Transport::new(tcp_config.clone()) as tcp::Transport<tcp::async_io::Tcp>
                );

        desktop_trans
    };

    let (transport, bandwith) = bandwidth::BandwidthLogging::new(transport);

    let authentication_config = {
        let noise_keypair = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&local_key)
            .expect("generating noise keypair");

        let noise_legacy = noise::LegacyConfig {
            recv_legacy_handshake: true,
            ..Default::default()
        };

        let mut xx_config = noise::NoiseConfig::xx(noise_keypair);
        xx_config.set_legacy_config(noise_legacy);
        xx_config.into_authenticated()
    };

    let multiplexing_config = {
        let mut mplex_config = mplex::MplexConfig::new();
        mplex_config.set_max_buffer_behaviour(mplex::MaxBufferBehaviour::Block);
        mplex_config.set_max_buffer_size(usize::MAX);

        let yamux_max_buffer_size: usize = 1024 * 1024;
        let mut yamux_config = yamux::YamuxConfig::default();
        yamux_config.set_window_update_mode(yamux::WindowUpdateMode::on_read());
        yamux_config.set_max_buffer_size(yamux_max_buffer_size);

        core::upgrade::SelectUpgrade::new(yamux_config, mplex_config)
    };

    let transport: Boxed<(PeerId, StreamMuxerBox)> = transport
        .upgrade(core::upgrade::Version::V1Lazy)
        .authenticate(authentication_config)
        .multiplex(multiplexing_config)
        .timeout(Duration::from_secs(5))
        .boxed();

    (transport, bandwith)
}
