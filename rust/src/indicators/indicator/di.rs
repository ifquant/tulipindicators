use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, triple_input};
use crate::indicators::shared::{true_range, DirectionalIndexState};
use crate::state::{IndicatorState, RingHistory};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "di",
    full_name: "Directional Indicator",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close"],
    option_names: &["period"],
    output_names: &["plus_di", "minus_di"],
};

#[derive(Debug, Clone, Copy)]
pub struct Di;

impl Di {
    pub fn state(options: &[Real], history_capacity: usize) -> Result<DiState, IndicatorError> {
        DiState::new(options, history_capacity)
    }
}

impl Indicator for Di {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period.saturating_sub(1))
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = high.len().saturating_sub(period.saturating_sub(1));
        if output_len == 0 {
            return Ok(vec![Vec::new(), Vec::new()]);
        }

        let mut plus = vec![0.0; output_len];
        let mut minus = vec![0.0; output_len];
        let written = run_di_batch(high, low, close, period, &mut plus, &mut minus);
        debug_assert_eq!(written, output_len);
        Ok(vec![plus, minus])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = high.len().saturating_sub(period.saturating_sub(1));
        validate_output_slices(&METADATA, outputs, 2)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        ensure_output_len(&METADATA, outputs[1].len(), output_len, 1)?;
        let (plus_outputs, minus_outputs) = outputs.split_at_mut(1);
        Ok(run_di_batch(
            high,
            low,
            close,
            period,
            &mut plus_outputs[0][..output_len],
            &mut minus_outputs[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(DiStream::new(options)?)))
    }
}

fn run_di_batch(
    high: &[Real],
    low: &[Real],
    close: &[Real],
    period: usize,
    plus: &mut [Real],
    minus: &mut [Real],
) -> usize {
    if high.len() < period {
        return 0;
    }

    let per = (period.saturating_sub(1)) as Real / period as Real;
    let mut atr = 0.0;
    let mut dmup = 0.0;
    let mut dmdown = 0.0;

    for index in 1..period {
        atr += true_range(high[index], low[index], close[index - 1]);
        let (dp, dm) =
            directional_movement(high[index - 1], high[index], low[index - 1], low[index]);
        dmup += dp;
        dmdown += dm;
    }

    plus[0] = 100.0 * dmup / atr;
    minus[0] = 100.0 * dmdown / atr;
    let mut out_index = 1usize;

    for index in period..high.len() {
        atr = atr * per + true_range(high[index], low[index], close[index - 1]);
        let (dp, dm) =
            directional_movement(high[index - 1], high[index], low[index - 1], low[index]);
        dmup = dmup * per + dp;
        dmdown = dmdown * per + dm;
        plus[out_index] = 100.0 * dmup / atr;
        minus[out_index] = 100.0 * dmdown / atr;
        out_index += 1;
    }

    out_index
}

fn directional_movement(
    previous_high: Real,
    high: Real,
    previous_low: Real,
    low: Real,
) -> (Real, Real) {
    let up = high - previous_high;
    let down = previous_low - low;
    if up > down && up > 0.0 {
        (up, 0.0)
    } else if down > up && down > 0.0 {
        (0.0, down)
    } else {
        (0.0, 0.0)
    }
}

struct DiStream {
    progress: usize,
    state: DirectionalIndexState,
}

impl DiStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            progress: 0,
            state: DirectionalIndexState::new(parse_period(options)?),
        })
    }

    fn update_one(&mut self, high: Real, low: Real, close: Real) -> Option<(Real, Real)> {
        let output = self
            .state
            .feed(high, low, close)
            .map(|(up, down, atr)| (100.0 * up / atr, 100.0 * down / atr));
        self.progress += 1;
        output
    }
}

impl IndicatorStream for DiStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let mut plus = Vec::new();
        let mut minus = Vec::new();

        for index in 0..high.len() {
            if let Some((up, down)) = self.update_one(high[index], low[index], close[index]) {
                plus.push(up);
                minus.push(down);
            }
        }

        Ok(vec![plus, minus])
    }
}

pub struct DiState {
    period: usize,
    stream: DiStream,
    history: RingHistory<(Real, Real)>,
}

impl DiState {
    pub fn new(options: &[Real], history_capacity: usize) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            period,
            stream: DiStream::new(options)?,
            history: RingHistory::new(history_capacity),
        })
    }
}

impl IndicatorState for DiState {
    type Input = (Real, Real, Real);
    type Output = (Real, Real);

    fn update(&mut self, input: Self::Input) -> Option<Self::Output> {
        let output = self.stream.update_one(input.0, input.1, input.2);
        if let Some(value) = output {
            self.history.push(value);
        }
        output
    }

    fn latest(&self) -> Option<Self::Output> {
        self.history.latest().cloned()
    }

    fn get(&self, index_from_latest: usize) -> Option<Self::Output> {
        self.history.get(index_from_latest).cloned()
    }

    fn len(&self) -> usize {
        self.history.len()
    }

    fn history_capacity(&self) -> usize {
        self.history.capacity()
    }

    fn reset(&mut self) {
        self.stream = DiStream {
            progress: 0,
            state: DirectionalIndexState::new(self.period),
        };
        self.history.clear();
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}
