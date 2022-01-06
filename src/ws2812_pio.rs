#![feature(generic_arg_infer)]

use super::ledstrip::{LEDColor, LEDStrip};
use super::{bsp, pio};
use bsp::hal::pio::{
    InstalledProgram, PIOBuilder, PIOExt, Running, StateMachine, StateMachineIndex, Stopped, Tx,
    UninitStateMachine, PIO, SM0,
};

pub struct WS2812PIO<P: PIOExt> {
    txfifo: bsp::hal::pio::Tx<(P, SM0)>,
    _sm: StateMachine<(P, SM0), Running>,
}

impl<P: PIOExt> WS2812PIO<P> {
    pub fn new(pio: P, resets: &mut bsp::hal::pac::RESETS, clk_freq: f32, pin: u8) -> Self {
        let (mut pioo, sm0, _, _, _) = pio.split(resets);
        let installed = pioo.install(&pio_program()).unwrap();
        let (sm, txfifo) = make_statemachine(&mut pioo, sm0, clk_freq, pin, installed);
        Self {
            _sm: sm.start(),
            txfifo,
        }
    }

    pub fn output<const NLED: usize>(&mut self, strip: &LEDStrip<NLED>) {
        for c in strip.iter() {
            while self.txfifo.is_full() {}
            self.txfifo.write(c);
        }

        while !self.txfifo.is_empty() {}
    }
}

fn pio_program() -> pio::Program<32> {
    let mut a = pio::Assembler::<32>::new_with_side_set(pio::SideSet::new(true, 1, false));
    let mut wrap_target = a.label();
    let mut wrap_source = a.label();
    let mut is_zero = a.label();
    a.set(pio::SetDestination::PINDIRS, 1);
    a.pull(false, true);
    a.bind(&mut wrap_target);
    a.out_with_delay_and_side_set(pio::OutDestination::X, 1, 2, 0);
    a.jmp_with_delay_and_side_set(pio::JmpCondition::XIsZero, &mut is_zero, 2, 1);
    a.jmp_with_delay_and_side_set(pio::JmpCondition::Always, &mut wrap_target, 3, 1);
    a.bind(&mut is_zero);
    a.nop_with_delay_and_side_set(3, 0);
    a.bind(&mut wrap_source);

    a.assemble_with_wrap(wrap_source, wrap_target)
}

fn make_statemachine<SM: StateMachineIndex, P: PIOExt>(
    pio: &mut PIO<P>,
    sm: UninitStateMachine<(P, SM)>,
    clk_freq: f32,
    pin: u8,
    installed_program: InstalledProgram<P>,
) -> (StateMachine<(P, SM), Stopped>, Tx<(P, SM)>) {
    let (sm, _, tx) = PIOBuilder::from_program(installed_program)
        .out_pins(pin, 1)
        .set_pins(pin, 1)
        .clock_divisor(clk_freq / (10. * 800000.))
        .side_set_pin_base(pin)
        .pull_threshold(24)
        .autopull(true)
        .out_shift_direction(bsp::hal::pio::ShiftDirection::Left)
        .buffers(bsp::hal::pio::Buffers::OnlyTx)
        .build(sm);
    (sm, tx)
}
