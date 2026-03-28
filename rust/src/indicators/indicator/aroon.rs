use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count, parse_usize_option};
use crate::indicators::shared::{ExtremaKind, MonotonicQueue};

const AROON_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "aroon",
    full_name: "Aroon",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low"],
    option_names: &["period"],
    output_names: &["aroon_down", "aroon_up"],
};

const AROONOSC_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "aroonosc",
    full_name: "Aroon Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low"],
    option_names: &["period"],
    output_names: &["aroonosc"],
};

#[derive(Debug, Clone, Copy)]
pub struct Aroon;

#[derive(Debug, Clone, Copy)]
pub struct AroonOsc;

impl Indicator for Aroon {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &AROON_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        parse_period(options)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = AroonStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(AroonStream::new(options)?)))
    }
}

impl Indicator for AroonOsc {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &AROONOSC_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        parse_period(options)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = AroonOscStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(AroonOscStream::new(options)?)))
    }
}

struct AroonStream {
    period: usize,
    progress: usize,
    max_queue: MonotonicQueue,
    min_queue: MonotonicQueue,
}

impl AroonStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(options)?,
            progress: 0,
            max_queue: MonotonicQueue::new(ExtremaKind::Max),
            min_queue: MonotonicQueue::new(ExtremaKind::Min),
        })
    }
}

impl IndicatorStream for AroonStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &AROON_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(AROON_METADATA.name, inputs)?;
        let mut down = Vec::with_capacity(high.len().saturating_sub(self.period));
        let mut up = Vec::with_capacity(high.len().saturating_sub(self.period));
        let scale = 100.0 / self.period as Real;

        for (&high, &low) in high.iter().zip(low.iter()) {
            let index = self.progress;
            self.max_queue.push(index, high);
            self.min_queue.push(index, low);

            if index >= self.period {
                let window_start = index - self.period;
                self.max_queue.evict_before(window_start);
                self.min_queue.evict_before(window_start);
                down.push(
                    (self.period as Real - (index - self.min_queue.front_index()) as Real) * scale,
                );
                up.push(
                    (self.period as Real - (index - self.max_queue.front_index()) as Real) * scale,
                );
            }

            self.progress += 1;
        }

        Ok(vec![down, up])
    }
}

struct AroonOscStream {
    period: usize,
    progress: usize,
    max_queue: MonotonicQueue,
    min_queue: MonotonicQueue,
}

impl AroonOscStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(options)?,
            progress: 0,
            max_queue: MonotonicQueue::new(ExtremaKind::Max),
            min_queue: MonotonicQueue::new(ExtremaKind::Min),
        })
    }
}

impl IndicatorStream for AroonOscStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &AROONOSC_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(AROONOSC_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len().saturating_sub(self.period));
        let scale = 100.0 / self.period as Real;

        for (&high, &low) in high.iter().zip(low.iter()) {
            let index = self.progress;
            self.max_queue.push(index, high);
            self.min_queue.push(index, low);

            if index >= self.period {
                let window_start = index - self.period;
                self.max_queue.evict_before(window_start);
                self.min_queue.evict_before(window_start);
                output.push(
                    (self.max_queue.front_index() as Real - self.min_queue.front_index() as Real)
                        * scale,
                );
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(AROON_METADATA.name, options, 1)?;
    parse_usize_option(AROON_METADATA.name, options, 0, "period", 1)
}
