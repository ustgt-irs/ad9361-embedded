//! Driver configuration of the ROMEO X-Band board, with manual gain control.
use core::num::NonZero;

use ad9361_embedded::{
    ReceiverId, RxPhaseConfig, TransmitterId,
    clocks::{
        BbClockPathConfigHelper, BbPllConfig, ClockConfig, Clocks, LocalOscClockConfig,
        RateGovernor, RefClockScalers, RxFirDecimation, TxFirInterpolation,
    },
    config::{
        AuxAdcConfig, AuxAdcDecimationBits, AuxDacConfig, ConfigRaw, ConfigValidated,
        DecPowerMeasurementDuration, DigitalInterfaceConfig, DivisionDuplexConfig,
        ExternalClockConfig, GainControl, GainControlCommon, GainControlManualConfig, GpoConfig,
        LvdsConfig, ParallelPortModeConfig, Pp1Register, Pp2Register, RssiConfig,
        RxClockDelayConfig, RxGainTableType, RxRfPathConfig, SingleGpoConfig, TempSenseConfig,
        TransceiverChannelMode, TxClockDelayConfig, TxMonitorConfig, TxRfPathConfig,
    },
    regs::{ClockScaler, DecPowerMeasurementSource},
};
use arbitrary_int::{traits::Integer as _, u2, u3, u4, u6, u13};

const REF_CLOCK_HZ: u32 = 40_000_000;
const LO_FREQUENCY_HZ: u64 = 2_400_000_000;
const TX_SAMPLE_HZ: u32 = 30_720_000;

pub fn build() -> ConfigValidated {
    let ref_clk_scalers = RefClockScalers {
        bb_refclk: ClockScaler::Div1,
        rx_synth: ClockScaler::Mul2,
        tx_synth: ClockScaler::Mul2,
    };
    let synth_clock_tx = ref_clk_scalers.calculate_tx_synth_clock(REF_CLOCK_HZ);
    let synth_clock_rx = ref_clk_scalers.calculate_rx_synth_clock(REF_CLOCK_HZ);
    let bb_pll_ref_clock = ref_clk_scalers.calculate_bb_pll_synth_clock(REF_CLOCK_HZ);

    let path_helper = BbClockPathConfigHelper::calculate_and_validate(
        TX_SAMPLE_HZ,
        false,
        RxFirDecimation::Div4EnableFilter,
        TxFirInterpolation::Mult4EnableFilter,
        RateGovernor::Nominal,
    )
    .expect("BB PLL clock calculation failed");
    let clock_config = ClockConfig {
        ref_clk_scalers,
        bb_pll: BbPllConfig::calculate(bb_pll_ref_clock, path_helper.bb_pll_clock_hz()),
        adc: path_helper.adc_div(),
        rx: path_helper.rx_config(
            LocalOscClockConfig::calculate_for_internal_lo(synth_clock_rx, LO_FREQUENCY_HZ)
                .unwrap(),
        ),
        tx: path_helper.tx_config(
            LocalOscClockConfig::calculate_for_internal_lo(synth_clock_tx, LO_FREQUENCY_HZ)
                .unwrap(),
        ),
    };
    let clocks = Clocks::new(REF_CLOCK_HZ, &clock_config);

    let gain_control = GainControl::new(
        ad9361_embedded::GainControlMode::Manual,
        ad9361_embedded::GainControlMode::Manual,
        GainControlCommon {
            digital_config: None,
            adc_ovr_sample_size: 1,
            dec_pow_measurement_duration: DecPowerMeasurementDuration::new(u4::new(5)),
            low_power_thresh: 24,
            dec_pwr_meas_source: DecPowerMeasurementSource::Hb1Out,
            gain_update_interval_us: 1000,
            adc_small_overload_thresh: 47,
            adc_large_overload_thresh: 58,
            lmt_overload_low_thresh_mv_peak: 704,
            lmt_overload_high_thresh_mv_peak: 800,
        },
        Some(GainControlManualConfig {
            dec_gain_step: 1,
            inc_gain_step: 1,
            rx1_ctrl_input: false,
            rx2_ctrl_input: false,
            split_table_mode: None,
        }),
        None,
        None,
    )
    .expect("invalid gain control config");

    ConfigRaw {
        reference_clk_rate: REF_CLOCK_HZ,
        rf_rx_bandwidth_hz: 40_864_500,
        rf_tx_bandwidth_hz: 20_000_000,
        clock: clock_config,
        transceiver_channel_mode: TransceiverChannelMode::Single {
            rx: ReceiverId::Rx1,
            tx: TransmitterId::Tx1,
        },
        digital_interface: DigitalInterfaceConfig {
            pp1_config: Pp1Register::default().with_rx_frame_pulse_mode(true),
            pp2_config: Pp2Register::default().with_delay_rx_data(u2::ZERO),
            pp3_config: ParallelPortModeConfig::new_for_lvds(),
            rx_default_delay: RxClockDelayConfig::builder()
                .with_data_clk_delay(u4::ZERO)
                .with_rx_data_delay(u4::new(4))
                .build(),
            tx_default_delay: TxClockDelayConfig::builder()
                .with_fb_clk_delay(u4::new(7))
                .with_tx_data_delay(u4::new(0))
                .build(),
            lvds_config: Some(LvdsConfig::new(150, true)),
            rx1rx2_phase_inversion: false,
        },
        rx_path: RxRfPathConfig::Diff_Rx1A_NP_Rx2A_NP,
        tx_path: TxRfPathConfig::PortA,
        division_duplex_config: DivisionDuplexConfig::Fdd {
            independent_mode: false,
        },
        external_clock_config: ExternalClockConfig::Dcxo {
            coarse_tune: u6::new(8),
            fine_tune: u13::new(5920),
        },
        gain_table_type: RxGainTableType::Full,
        clkout_mode: None,
        ensm_enable_pin_pulse_mode: false,
        ensm_enable_txnrx_control: false,
        tx_attenuation_md_b: 10000,
        update_tx_gain_in_alert: false,
        gain_control,
        rssi: RssiConfig::default(),
        aux_adc: AuxAdcConfig {
            decimation: AuxAdcDecimationBits::new(u3::new(1)),
            clock_rate_hz: NonZero::new(clocks.tx().dac_clk()).expect("DAC clock is zero"),
            temp_sense: TempSenseConfig {
                decimation: AuxAdcDecimationBits::new(u3::new(1)),
                measurement_interval_ms: 100,
                offset_signed: 0,
                enable_periodic: false,
            },
        },
        aux_dac: AuxDacConfig {
            aux_dac_manual_mode_enable: true,
            dac_config: [None, None],
        },
        ctrl_outs_enable_mask: 0,
        ctrl_outs_index: 0,
        elna: None,
        gpo_config: GpoConfig {
            manual_mode_enable: true,
            gpo: [SingleGpoConfig::default(); 4],
        },
        tx_quad_calib: Some(RxPhaseConfig::Calculated),
        tx_monitoring: TxMonitorConfig::Disabled,
        dc_offset_tracking_update_event: u3::ZERO,
        dc_offset_attenuation_high_range: 6,
        dc_offset_attenuation_low_range: 5,
        dc_offset_count_high_range: 0x28,
        dc_offset_count_low_range: 0x32,
        qec_tracking_slow_mode: false,
    }
    .validate()
    .expect("invalid AD9361 configuration")
}
