use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, triple_input};
use crate::indicators::shared::{ExtremaKind, MonotonicQueue};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "willr",
    full_name: "Williams %R",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close"],
    option_names: &["period"],
    output_names: &["willr"],
};

#[derive(Debug, Clone, Copy)]
pub struct WillR;

impl Indicator for WillR {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let lookback = period - 1;
        let mut output = Vec::with_capacity(high.len().saturating_sub(lookback));

        if high.len() <= lookback {
            return Ok(vec![output]);
        }

        let mut max_queue = MonotonicQueue::new(ExtremaKind::Max);
        let mut min_queue = MonotonicQueue::new(ExtremaKind::Min);

        for index in 0..high.len() {
            max_queue.push(index, high[index]);
            min_queue.push(index, low[index]);

            if index + 1 >= period {
                let window_start = index + 1 - period;
                max_queue.evict_before(window_start);
                min_queue.evict_before(window_start);
                output.push(willr_value(
                    max_queue.front_value(),
                    min_queue.front_value(),
                    close[index],
                ));
            }
        }

        Ok(vec![output])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(WillRStream::new(options)?)))
    }
}

struct WillRStream {
    period: usize,
    progress: usize,
    max_queue: MonotonicQueue,
    min_queue: MonotonicQueue,
}

impl WillRStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(options)?,
            progress: 0,
            max_queue: MonotonicQueue::new(ExtremaKind::Max),
            min_queue: MonotonicQueue::new(ExtremaKind::Min),
        })
    }
}

impl IndicatorStream for WillRStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len());

        for local_index in 0..high.len() {
            let index = self.progress;
            self.max_queue.push(index, high[local_index]);
            self.min_queue.push(index, low[local_index]);

            if index + 1 >= self.period {
                let window_start = index + 1 - self.period;
                self.max_queue.evict_before(window_start);
                self.min_queue.evict_before(window_start);
                output.push(willr_value(
                    self.max_queue.front_value(),
                    self.min_queue.front_value(),
                    close[local_index],
                ));
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}

fn willr_value(max: Real, min: Real, close: Real) -> Real {
    let highlow = max - min;
    if highlow == 0.0 {
        0.0
    } else {
        -100.0 * ((max - close) / highlow)
    }
}
