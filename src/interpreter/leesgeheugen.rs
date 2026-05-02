use std::collections::{BTreeMap, VecDeque};
use crate::EcolMachine;
use crate::interpreter::errors::EcolFout;
use crate::interpreter::program::LineInhoud;

pub struct LeesGeheugen {
    lees_hervat_bij: Option<u16>,
    lees_buffer: VecDeque<f32>,
    leessym_hervat_bij: Option<u16>,
    leessym_buffer: VecDeque<u8>,
    lopende_machine: Option<EcolMachine>,
    lopend_programma: Option<BTreeMap<u16, LineInhoud>>,
    wacht_op_laad: bool,
}

impl LeesGeheugen {
    pub fn new() -> Self {
        LeesGeheugen {
            lees_hervat_bij: None,
            lees_buffer: VecDeque::new(),
            leessym_hervat_bij: None,
            leessym_buffer: VecDeque::new(),
            lopende_machine: None,
            lopend_programma: None,
            wacht_op_laad: false,
        }
    }
    pub fn wacht_op_lees(&self) -> bool {
        self.lees_hervat_bij.is_some()
    }
    pub(super) fn lees_hervat_bij(&mut self) -> Option<u16> {
        self.lees_hervat_bij
    }
    pub(super) fn lees_hervat_none(&mut self)  {
        self.lees_hervat_bij = None;
    }
    pub(super) fn lees_hervat_bij_op_regel(&mut self, regelnummer: u16)  {
        self.lees_hervat_bij = Some(regelnummer);
    }
    pub(super) fn lees_waarde(&mut self) -> Option<f32> {
        self.lees_buffer.pop_front()
    }
    pub(super) fn schrijf_lees_waarde(&mut self, waarde: f32) {
        self.lees_buffer.push_back(waarde);
    }
    pub fn wacht_op_leessym(&self) -> bool {
        self.leessym_hervat_bij.is_some()
    }
    pub(super) fn leessym_hervat_bij(&mut self) -> Option<u16> {
        self.leessym_hervat_bij
    }
    pub(super) fn leessym_hervat_none(&mut self)  {
        self.leessym_hervat_bij = None;
    }
    pub(super) fn leessym_hervat_bij_op_regel(&mut self, regelnummer: u16)  {
        self.leessym_hervat_bij = Some(regelnummer);
    }
    pub(super) fn leessym_waarde(&mut self) -> Option<f32> {
        self.leessym_buffer.pop_front().map(f32::from)
    }
    pub(super) fn schrijf_leessym_waarde(&mut self, waarde: f32) -> Result<(), EcolFout> {
        if waarde.fract() != 0.0 || !(0f32..=99f32).contains(&waarde) {
            return Err(EcolFout::FoutMelding(format!("Waarde {waarde} is ongeldig (xxxSYM verwacht een geheel getal 0–99).")));
        }
        
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        self.leessym_buffer.push_back(waarde as u8); //veilig, want hierboven gevalideerd als geheel getal tussen 0 en 99
        Ok(())
    }
    pub fn wacht_op_laad(&self) -> bool {
        self.wacht_op_laad
    }
    pub fn reset_laad(&mut self) {
        self.wacht_op_laad = false;
    }
    pub(super) fn stel_laad_in(&mut self) {
        self.wacht_op_laad = true;
    }
    pub(super) fn neem_lopende_toestand(&mut self) -> Option<(EcolMachine, BTreeMap<u16, LineInhoud>)> {
        match (self.lopende_machine.take(), self.lopend_programma.take()) {
            (Some(m), Some(p)) => Some((m, p)),
            _ => None,
        }
    }
    pub(super) fn sla_lopende_toestand_op(&mut self, machine: EcolMachine, programma: BTreeMap<u16, LineInhoud>) {
        self.lopende_machine = Some(machine);
        self.lopend_programma = Some(programma);
    }
}
