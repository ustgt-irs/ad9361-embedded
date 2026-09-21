use embedded_hal::digital::InputPin;

pub struct ControlOutputReader<const REG: usize> {
    raw: u8,
    enabled: u8,
}

impl<const REG: usize> ControlOutputReader<REG> {
    pub const fn new(raw: u8, enabled: u8) -> Self {
        Self { raw, enabled }
    }

    pub const fn checked_bit(&self, pos: u8) -> Option<bool> {
        if ((self.enabled >> pos) & 1) == 0 {
            return None;
        }
        Some((self.raw >> pos) & 1 == 1)
    }

    pub const fn update_raw(&mut self, raw: u8) {
        self.raw = raw;
    }

    pub const fn update_bit(&mut self, pos: u8, value: bool) {
        if value {
            self.raw |= 1 << pos;
        } else {
            self.raw &= !(1 << pos);
        }
    }

    pub const fn update_d0(&mut self, value: bool) {
        self.update_bit(0, value);
    }
    pub const fn update_d1(&mut self, value: bool) {
        self.update_bit(1, value);
    }
    pub const fn update_d2(&mut self, value: bool) {
        self.update_bit(2, value);
    }
    pub const fn update_d3(&mut self, value: bool) {
        self.update_bit(3, value);
    }
    pub const fn update_d4(&mut self, value: bool) {
        self.update_bit(4, value);
    }
    pub const fn update_d5(&mut self, value: bool) {
        self.update_bit(5, value);
    }
    pub const fn update_d6(&mut self, value: bool) {
        self.update_bit(6, value);
    }
    pub const fn update_d7(&mut self, value: bool) {
        self.update_bit(7, value);
    }

    pub fn update_from_pins<P, E>(&mut self, pins: &mut [P; 8]) -> Result<(), E>
    where
        P: InputPin<Error = E>,
    {
        let mut raw = 0u8;

        let mut i = 0;
        while i < 8 {
            if pins[i].is_high()? {
                raw |= 1 << i;
            }
            i += 1;
        }

        self.raw = raw;
        Ok(())
    }
}

pub type CtrlOutReader0x00 = ControlOutputReader<0x00>;
pub type CtrlOutReader0x01 = ControlOutputReader<0x01>;
pub type CtrlOutReader0x02 = ControlOutputReader<0x02>;
pub type CtrlOutReader0x03 = ControlOutputReader<0x03>;
pub type CtrlOutReader0x04 = ControlOutputReader<0x04>;
pub type CtrlOutReader0x05 = ControlOutputReader<0x05>;
pub type CtrlOutReader0x06 = ControlOutputReader<0x06>;
pub type CtrlOutReader0x07 = ControlOutputReader<0x07>;
pub type CtrlOutReader0x08 = ControlOutputReader<0x08>;
pub type CtrlOutReader0x09 = ControlOutputReader<0x09>;
pub type CtrlOutReader0x0A = ControlOutputReader<0x0A>;
pub type CtrlOutReader0x0B = ControlOutputReader<0x0B>;
pub type CtrlOutReader0x0C = ControlOutputReader<0x0C>;
pub type CtrlOutReader0x0D = ControlOutputReader<0x0D>;
pub type CtrlOutReader0x0E = ControlOutputReader<0x0E>;
pub type CtrlOutReader0x0F = ControlOutputReader<0x0F>;
pub type CtrlOutReader0x10 = ControlOutputReader<0x10>;
pub type CtrlOutReader0x11 = ControlOutputReader<0x11>;
pub type CtrlOutReader0x12 = ControlOutputReader<0x12>;
pub type CtrlOutReader0x13 = ControlOutputReader<0x13>;
pub type CtrlOutReader0x14 = ControlOutputReader<0x14>;
pub type CtrlOutReader0x15 = ControlOutputReader<0x15>;
pub type CtrlOutReader0x16 = ControlOutputReader<0x16>;
pub type CtrlOutReader0x17 = ControlOutputReader<0x17>;
pub type CtrlOutReader0x18 = ControlOutputReader<0x18>;
pub type CtrlOutReader0x19 = ControlOutputReader<0x19>;
pub type CtrlOutReader0x1A = ControlOutputReader<0x1A>;
pub type CtrlOutReader0x1B = ControlOutputReader<0x1B>;
pub type CtrlOutReader0x1C = ControlOutputReader<0x1C>;
pub type CtrlOutReader0x1D = ControlOutputReader<0x1D>;
pub type CtrlOutReader0x1E = ControlOutputReader<0x1E>;
pub type CtrlOutReader0x1F = ControlOutputReader<0x1F>;

impl ControlOutputReader<0x0> {
    pub const fn calibration_done(&self) -> Option<bool> {
        self.checked_bit(7)
    }

    pub const fn tx_cp_cal_done(&self) -> Option<bool> {
        self.checked_bit(6)
    }

    pub const fn rx_cp_cal_done(&self) -> Option<bool> {
        self.checked_bit(5)
    }

    pub const fn rx_bb_filter_tuning_done(&self) -> Option<bool> {
        self.checked_bit(4)
    }

    pub const fn tx_bb_filter_tuning_done(&self) -> Option<bool> {
        self.checked_bit(3)
    }

    pub const fn gain_step_cal_busy(&self) -> Option<bool> {
        self.checked_bit(2)
    }

    pub const fn rx_synth_vco_cal_busy(&self) -> Option<bool> {
        self.checked_bit(1)
    }

    pub const fn tx_synth_vco_cal_busy(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x01> {
    pub const fn tx_rf_pll_lock(&self) -> Option<bool> {
        self.checked_bit(7)
    }

    pub const fn rx_rf_pll_lock(&self) -> Option<bool> {
        self.checked_bit(6)
    }

    pub const fn bbpll_lock(&self) -> Option<bool> {
        self.checked_bit(5)
    }
}

impl ControlOutputReader<0x02> {
    pub const fn bb_dc_cal_busy(&self) -> Option<bool> {
        self.checked_bit(7)
    }

    pub const fn rf_dc_cal_busy(&self) -> Option<bool> {
        self.checked_bit(6)
    }

    pub const fn ch1_rx_quad_cal_busy(&self) -> Option<bool> {
        self.checked_bit(5)
    }

    pub const fn ch1_tx_quad_cal_busy(&self) -> Option<bool> {
        self.checked_bit(4)
    }

    pub const fn ch2_rx_quad_cal_busy(&self) -> Option<bool> {
        self.checked_bit(3)
    }

    pub const fn ch2_tx_quad_cal_busy(&self) -> Option<bool> {
        self.checked_bit(2)
    }

    pub const fn gain_step_cal_busy(&self) -> Option<bool> {
        self.checked_bit(1)
    }

    pub const fn tx_mon_cal_busy(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x03> {
    pub const fn ch1_adc_low_power(&self) -> Option<bool> {
        self.checked_bit(7)
    }

    pub const fn ch1_lg_lmt_ovrg(&self) -> Option<bool> {
        self.checked_bit(6)
    }

    pub const fn ch1_lg_adc_ovrg(&self) -> Option<bool> {
        self.checked_bit(5)
    }

    pub const fn ch1_sm_adc_ovrg(&self) -> Option<bool> {
        self.checked_bit(4)
    }

    pub const fn ch2_low_power(&self) -> Option<bool> {
        self.checked_bit(3)
    }

    pub const fn ch2_lg_lmt_ovrg(&self) -> Option<bool> {
        self.checked_bit(2)
    }

    pub const fn ch2_lg_adc_ovrg(&self) -> Option<bool> {
        self.checked_bit(1)
    }

    pub const fn ch2_sm_adc_ovrg(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x04> {
    pub const fn ch2_rx_gain_6(&self) -> Option<bool> {
        self.checked_bit(7)
    }

    pub const fn ch2_rx_gain_5(&self) -> Option<bool> {
        self.checked_bit(6)
    }

    pub const fn ch2_rx_gain_4(&self) -> Option<bool> {
        self.checked_bit(5)
    }

    pub const fn ch2_rx_gain_3(&self) -> Option<bool> {
        self.checked_bit(4)
    }

    pub const fn ch2_rx_gain_2(&self) -> Option<bool> {
        self.checked_bit(3)
    }

    pub const fn ch2_lg_lmt_ovrg(&self) -> Option<bool> {
        self.checked_bit(2)
    }

    pub const fn ch2_lg_adc_ovrg(&self) -> Option<bool> {
        self.checked_bit(1)
    }

    pub const fn ch2_gain_lock(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x05> {
    pub const fn ch2_gain_change(&self) -> Option<bool> {
        self.checked_bit(7)
    }

    pub const fn ch1_gain_change(&self) -> Option<bool> {
        self.checked_bit(6)
    }

    pub const fn ch2_low_power(&self) -> Option<bool> {
        self.checked_bit(5)
    }

    pub const fn ch2_lg_lmt_ovrg(&self) -> Option<bool> {
        self.checked_bit(4)
    }

    pub const fn ch2_lg_adc_ovrg(&self) -> Option<bool> {
        self.checked_bit(3)
    }

    pub const fn ch2_gain_lock(&self) -> Option<bool> {
        self.checked_bit(2)
    }

    pub const fn ch2_energy_lost(&self) -> Option<bool> {
        self.checked_bit(1)
    }

    pub const fn ch2_stronger_signal(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x06> {
    pub const fn ch1_low_power(&self) -> Option<bool> {
        self.checked_bit(7)
    }

    pub const fn ch1_lg_lmt_ovrg(&self) -> Option<bool> {
        self.checked_bit(6)
    }

    pub const fn ch1_lg_adc_ovrg(&self) -> Option<bool> {
        self.checked_bit(5)
    }

    pub const fn ch1_rx_gain_6(&self) -> Option<bool> {
        self.checked_bit(4)
    }

    pub const fn ch1_rx_gain_5(&self) -> Option<bool> {
        self.checked_bit(3)
    }

    pub const fn ch1_rx_gain_4(&self) -> Option<bool> {
        self.checked_bit(2)
    }

    pub const fn ch1_rx_gain_3(&self) -> Option<bool> {
        self.checked_bit(1)
    }

    pub const fn ch1_rx_gain_2(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x07> {
    pub const fn ch1_low_power(&self) -> Option<bool> {
        self.checked_bit(7)
    }

    pub const fn ch1_lg_lmt_ovrg(&self) -> Option<bool> {
        self.checked_bit(6)
    }

    pub const fn ch1_lg_adc_ovrg(&self) -> Option<bool> {
        self.checked_bit(5)
    }

    pub const fn ch1_sm_adc_ovrg(&self) -> Option<bool> {
        self.checked_bit(4)
    }

    pub const fn ch1_agc_sm_2(&self) -> Option<bool> {
        self.checked_bit(3)
    }

    pub const fn ch1_agc_sm_1(&self) -> Option<bool> {
        self.checked_bit(2)
    }

    pub const fn ch1_agc_sm_0(&self) -> Option<bool> {
        self.checked_bit(1)
    }

    pub const fn ch1_gain_lock(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x08> {
    pub const fn ch1_stronger_signal(&self) -> Option<bool> {
        self.checked_bit(7)
    }

    pub const fn ch1_gain_lock(&self) -> Option<bool> {
        self.checked_bit(6)
    }

    pub const fn ch1_energy_lost(&self) -> Option<bool> {
        self.checked_bit(5)
    }

    pub const fn ch1_gain_change(&self) -> Option<bool> {
        self.checked_bit(4)
    }

    pub const fn ch2_stronger_signal(&self) -> Option<bool> {
        self.checked_bit(3)
    }

    pub const fn ch2_gain_lock(&self) -> Option<bool> {
        self.checked_bit(2)
    }

    pub const fn ch2_energy_lost(&self) -> Option<bool> {
        self.checked_bit(1)
    }

    pub const fn ch2_gain_change(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x09> {
    pub const fn rx_on(&self) -> Option<bool> {
        self.checked_bit(7)
    }

    pub const fn ch1_rssi_preamble_ready(&self) -> Option<bool> {
        self.checked_bit(6)
    }

    pub const fn ch1_rssi_symbol_ready(&self) -> Option<bool> {
        self.checked_bit(5)
    }

    pub const fn tx_on(&self) -> Option<bool> {
        self.checked_bit(4)
    }

    pub const fn ch2_rssi_preamble_ready(&self) -> Option<bool> {
        self.checked_bit(3)
    }

    pub const fn ch2_rssi_symbol_ready(&self) -> Option<bool> {
        self.checked_bit(2)
    }
}

impl ControlOutputReader<0x0A> {
    pub const fn ch1_tx_int3_overflow(&self) -> Option<bool> {
        self.checked_bit(7)
    }

    pub const fn ch1_tx_hb3_overflow(&self) -> Option<bool> {
        self.checked_bit(6)
    }

    pub const fn ch1_tx_hb2_overflow(&self) -> Option<bool> {
        self.checked_bit(5)
    }

    pub const fn ch1_tx_qec_overflow(&self) -> Option<bool> {
        self.checked_bit(4)
    }

    pub const fn ch1_tx_hb1_overflow(&self) -> Option<bool> {
        self.checked_bit(3)
    }

    pub const fn ch1_tx_fir_overflow(&self) -> Option<bool> {
        self.checked_bit(2)
    }

    pub const fn ch1_rx_fir_overflow(&self) -> Option<bool> {
        self.checked_bit(1)
    }
}

impl ControlOutputReader<0x0B> {
    pub const fn cal_seq_state_3(&self) -> Option<bool> {
        self.checked_bit(7)
    }

    pub const fn cal_seq_state_2(&self) -> Option<bool> {
        self.checked_bit(6)
    }

    pub const fn cal_seq_state_1(&self) -> Option<bool> {
        self.checked_bit(5)
    }

    pub const fn cal_seq_state_0(&self) -> Option<bool> {
        self.checked_bit(4)
    }

    pub const fn ensm_3(&self) -> Option<bool> {
        self.checked_bit(3)
    }

    pub const fn ensm_2(&self) -> Option<bool> {
        self.checked_bit(2)
    }

    pub const fn ensm_1(&self) -> Option<bool> {
        self.checked_bit(1)
    }

    pub const fn ensm_0(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x0C> {
    pub const fn ch1_energy_lost(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn ch1_reset_peak_detect(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn ch2_energy_lost(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn ch2_reset_peak_detect(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn gain_freeze(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn ch1_digital_sat(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn ch2_digital_sat(&self) -> Option<bool> {
        self.checked_bit(1)
    }
}

impl ControlOutputReader<0x0D> {
    pub const fn ch1_tx_quad_cal_status_1(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn ch1_tx_quad_cal_status_0(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn ch1_tx_quad_cal_done(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn rf_dc_cal_busy(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn ch2_tx_quad_cal_status_1(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn ch2_tx_quad_cal_status_0(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn ch2_tx_quad_cal_done(&self) -> Option<bool> {
        self.checked_bit(1)
    }
}

impl ControlOutputReader<0x0E> {
    pub const fn bb_dc_cal_busy(&self) -> Option<bool> {
        self.checked_bit(4)
    }
}

impl ControlOutputReader<0x0F> {
    pub const fn ch1_agc_state_2(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn ch1_agc_state_1(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn ch1_agc_state_0(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn ch1_reset_peak_detect(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn ch2_reset_peak_detect(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn ch1_rf_dc_cal_state_1(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn ch1_rf_dc_cal_state_0(&self) -> Option<bool> {
        self.checked_bit(1)
    }
}

impl ControlOutputReader<0x10> {
    pub const fn ch2_agc_state_2(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn ch2_agc_state_1(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn ch2_agc_state_0(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn ch2_enable_rssi(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn ch1_enable_rssi(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn ch2_rf_dc_cal_state_1(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn ch2_rf_dc_cal_state_0(&self) -> Option<bool> {
        self.checked_bit(1)
    }
}

impl ControlOutputReader<0x11> {
    pub const fn auxadc_output_11(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn auxadc_output_10(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn auxadc_output_9(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn auxadc_output_8(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn auxadc_output_7(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn auxadc_output_6(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn auxadc_output_5(&self) -> Option<bool> {
        self.checked_bit(1)
    }
    pub const fn auxadc_output_4(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x12> {
    pub const fn ch1_filter_power_ready(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn ch1_gain_lock(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn ch1_energy_lost(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn ch1_stronger_signal(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn ch1_adc_power_ready(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn ch1_agc_state_2(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn ch1_agc_state_1(&self) -> Option<bool> {
        self.checked_bit(1)
    }
    pub const fn ch1_agc_state_0(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x13> {
    pub const fn ch2_filter_power_ready(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn ch2_gain_lock(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn ch2_energy_lost(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn ch2_stronger_signal(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn ch2_adc_power_ready(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn ch2_agc_state_2(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn ch2_agc_state_1(&self) -> Option<bool> {
        self.checked_bit(1)
    }
    pub const fn ch2_agc_state_0(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x14> {
    pub const fn ch2_tx_int3_overflow(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn ch2_tx_hb3_overflow(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn ch2_tx_hb2_overflow(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn ch2_tx_qec_overflow(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn ch2_tx_hb1_overflow(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn ch2_tx_fir_overflow(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn ch2_rx_fir_overflow(&self) -> Option<bool> {
        self.checked_bit(1)
    }
}

impl ControlOutputReader<0x15> {
    pub const fn ch1_soi_present(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn ch1_update_dcrf(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn ch1_measure_dcrf(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn ch1_dc_track_count_reached(&self) -> Option<bool> {
        self.checked_bit(4)
    }
}

impl ControlOutputReader<0x16> {
    pub const fn ch1_gain_lock(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn ch1_rx_gain_6(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn ch1_rx_gain_5(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn ch1_rx_gain_4(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn ch1_rx_gain_3(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn ch1_rx_gain_2(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn ch1_rx_gain_1(&self) -> Option<bool> {
        self.checked_bit(1)
    }
    pub const fn ch1_rx_gain_0(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x17> {
    pub const fn ch2_gain_lock(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn ch2_rx_gain_6(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn ch2_rx_gain_5(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn ch2_rx_gain_4(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn ch2_rx_gain_3(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn ch2_rx_gain_2(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn ch2_rx_gain_1(&self) -> Option<bool> {
        self.checked_bit(1)
    }
    pub const fn ch2_rx_gain_0(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x18> {
    pub const fn ch2_soi_present(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn ch2_update_dcrf(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn ch2_measure_dcrf(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn ch2_dc_track_count_reached(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn ch2_enable_dec_pwr(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn ch2_enable_adc_pwr(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn ch1_enable_dec_pwr(&self) -> Option<bool> {
        self.checked_bit(1)
    }
    pub const fn ch1_enable_adc_pwr(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x19> {
    pub const fn rx_syn_cp_cal_3(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn rx_syn_cp_cal_2(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn rx_syn_cp_cal_1(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn rx_syn_cp_cal_0(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn tx_syn_cp_cal_3(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn tx_syn_cp_cal_2(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn tx_syn_cp_cal_1(&self) -> Option<bool> {
        self.checked_bit(1)
    }
    pub const fn tx_syn_cp_cal_0(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x1A> {
    pub const fn rx_syn_vco_tuning_8(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn rx_synth_vco_alc_6(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn rx_synth_vco_alc_5(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn rx_synth_vco_alc_4(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn rx_synth_vco_alc_3(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn rx_synth_vco_alc_2(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn rx_synth_vco_alc_1(&self) -> Option<bool> {
        self.checked_bit(1)
    }
    pub const fn rx_synth_vco_alc_0(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x1B> {
    pub const fn tx_syn_vco_tuning_8(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn tx_synth_vco_alc_6(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn tx_synth_vco_alc_5(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn tx_synth_vco_alc_4(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn tx_synth_vco_alc_3(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn tx_synth_vco_alc_2(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn tx_synth_vco_alc_1(&self) -> Option<bool> {
        self.checked_bit(1)
    }
    pub const fn tx_synth_vco_alc_0(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x1C> {
    pub const fn rx_syn_vco_tuning_7(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn rx_syn_vco_tuning_6(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn rx_syn_vco_tuning_5(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn rx_syn_vco_tuning_4(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn rx_syn_vco_tuning_3(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn rx_syn_vco_tuning_2(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn rx_syn_vco_tuning_1(&self) -> Option<bool> {
        self.checked_bit(1)
    }
    pub const fn rx_syn_vco_tuning_0(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x1D> {
    pub const fn tx_syn_vco_tuning_7(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn tx_syn_vco_tuning_6(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn tx_syn_vco_tuning_5(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn tx_syn_vco_tuning_4(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn tx_syn_vco_tuning_3(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn tx_syn_vco_tuning_2(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn tx_syn_vco_tuning_1(&self) -> Option<bool> {
        self.checked_bit(1)
    }
    pub const fn tx_syn_vco_tuning_0(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x1E> {
    pub const fn ch1_low_thresh_exceeded(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn ch1_high_thresh_exceeded(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn ch1_gain_upd_count_exp(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn ch1_agc_state_1(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn ch1_agc_state_0(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn ch1_gain_change(&self) -> Option<bool> {
        self.checked_bit(2)
    }
    pub const fn temp_sense_valid(&self) -> Option<bool> {
        self.checked_bit(1)
    }
    pub const fn auxadc_valid(&self) -> Option<bool> {
        self.checked_bit(0)
    }
}

impl ControlOutputReader<0x1F> {
    pub const fn ch2_low_thresh_exceeded(&self) -> Option<bool> {
        self.checked_bit(7)
    }
    pub const fn ch2_high_thresh_exceeded(&self) -> Option<bool> {
        self.checked_bit(6)
    }
    pub const fn ch2_gain_upd_count_exp(&self) -> Option<bool> {
        self.checked_bit(5)
    }
    pub const fn ch2_agc_sm_1(&self) -> Option<bool> {
        self.checked_bit(4)
    }
    pub const fn ch2_agc_sm_0(&self) -> Option<bool> {
        self.checked_bit(3)
    }
    pub const fn ch2_gain_change(&self) -> Option<bool> {
        self.checked_bit(2)
    }
}
