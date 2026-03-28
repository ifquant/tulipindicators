use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::{ExtremaKind, MonotonicQueue, RsiState};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "stochrsi",
    full_name: "Stochastic Relative Strength Index",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["stochrsi"],
};

#[derive(Debug, Clone, Copy)]
pub struct StochRsi;

impl Indicator for StochRsi {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period * 2 - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let mut output = Vec::with_capacity(input.len().saturating_sub(period * 2 - 1));
        let mut rsi_progress = 0usize;
        let mut rsi_state = RsiState::new(period);
        let mut max_queue = MonotonicQueue::new(ExtremaKind::Max);
        let mut min_queue = MonotonicQueue::new(ExtremaKind::Min);

        for &sample in input {
            if let Some(rsi) = rsi_state.feed(sample) {
                max_queue.push(rsi_progress, rsi);
                min_queue.push(rsi_progress, rsi);

                let window_start = rsi_progress.saturating_sub(period - 1);
                max_queue.evict_before(window_start);
                min_queue.evict_before(window_start);

                if rsi_progress + 1 >= period {
                    let max = max_queue.front_value();
                    let min = min_queue.front_value();
                    let diff = max - min;
                    output.push(if diff == 0.0 { 0.0 } else { (rsi - min) / diff });
                }

                rsi_progress += 1;
            }
        }

        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = input.len().saturating_sub(period * 2 - 1);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;

        let mut produced = 0usize;
        let mut rsi_progress = 0usize;
        let mut rsi_state = RsiState::new(period);
        let mut max_queue = MonotonicQueue::new(ExtremaKind::Max);
        let mut min_queue = MonotonicQueue::new(ExtremaKind::Min);

        for &sample in input {
            if let Some(rsi) = rsi_state.feed(sample) {
                max_queue.push(rsi_progress, rsi);
                min_queue.push(rsi_progress, rsi);

                let window_start = rsi_progress.saturating_sub(period - 1);
                max_queue.evict_before(window_start);
                min_queue.evict_before(window_start);

                if rsi_progress + 1 >= period {
                    let max = max_queue.front_value();
                    let min = min_queue.front_value();
                    let diff = max - min;
                    outputs[0][produced] = if diff == 0.0 { 0.0 } else { (rsi - min) / diff };
                    produced += 1;
                }

                rsi_progress += 1;
            }
        }

        Ok(produced)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(StochRsiStream::new(options)?)))
    }
}

struct StochRsiStream {
    period: usize,
    progress: usize,
    rsi_progress: usize,
    rsi_state: RsiState,
    max_queue: MonotonicQueue,
    min_queue: MonotonicQueue,
}

impl StochRsiStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            period,
            progress: 0,
            rsi_progress: 0,
            rsi_state: RsiState::new(period),
            max_queue: MonotonicQueue::new(ExtremaKind::Max),
            min_queue: MonotonicQueue::new(ExtremaKind::Min),
        })
    }
}

impl IndicatorStream for StochRsiStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = Vec::new();

        for sample in input {
            if let Some(rsi) = self.rsi_state.feed(*sample) {
                self.max_queue.push(self.rsi_progress, rsi);
                self.min_queue.push(self.rsi_progress, rsi);

                let window_start = self.rsi_progress.saturating_sub(self.period - 1);
                self.max_queue.evict_before(window_start);
                self.min_queue.evict_before(window_start);

                if self.rsi_progress + 1 >= self.period {
                    let max = self.max_queue.front_value();
                    let min = self.min_queue.front_value();
                    let diff = max - min;
                    output.push(if diff == 0.0 { 0.0 } else { (rsi - min) / diff });
                }

                self.rsi_progress += 1;
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 2)
}
