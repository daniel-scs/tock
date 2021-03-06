Kernel Analog-to-Digital Conversion HIL
========================================

**TRD:** 102 <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Philip Levis <br/>
**Draft-Created:** Dec 18, 2016<br/>
**Draft-Modified:** Jan 30, 2017<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** tock-dev@googlegroups.com</br>

Abstract
-------------------------------

This document describes the hardware independent layer interface (HIL)
for analog-to-digital conversion in the Tock operating system kernel.
It describes the Rust traits and other definitions for this service
as well as the reasoning behind them. This document is in full compliance
with TRD1.

1. Introduction
-------------------------------

Analog-to-digital converters (ADCs) are devices that convert analog input
signals to discrete digital output signals, typically voltage to a binary
number. While different microcontrollers can have very different control
registers and operating modes, the basic high-level interface they provide
is very uniform. Software that wishes to use more advanced features should
directly use the per-chip implementations, which may export these features.

The ADC HIL is the kernel crate, in module hil::adc. It
provides three traits:

  * kernel::hil::adc::AdcSingle: takes a single reading from an ADC port
  * kernel::hil::adc::AdcContinuous: starts a continuous stream of readings from a port
  * kernek::hil::adc::Client: handles the callback when a sample is obtained

The rest of this document discusses each in turn.


2. AdcSingle trait
-------------------------------

The AdcSingle trait is for requesting a single ADC conversion. It has
three functions:


    pub trait AdcSingle {
        /// Initialize must be called before taking a sample.
        fn initialize(&self) -> Result;

        /// Request a single ADC sample on a particular channel.
        fn sample(&self, channel: u8) -> Result;

        /// Cancel an outstanding request. Returning SUCCESS
        /// means it was cancelled and there will be no callback
        /// invocation. Returning FAIL means it was not cancelled and
        /// a callback will be invoked.
        fn cancel_sample(&self) -> Result;
    }

The `initialize` function MUST be called at least once before any
samples are taken. It only needs to be called once, not once per sample
This function MUST return SUCCESS, ERESERVE, and FAIL.

The `sample` function starts a single conversion on the specified
ADC channel. The exact binding of this channel to external sensors or
other inputs is board-dependent, so must be known by higher-level
software. This function MUST return SUCCESS, FAIL, EBUSY, EOFF, ERESERVE,
or EINVAL.

The `cancel_sample` function may be used to try to cancel an outstanding
conversion request. Because the conversion may have already begun, or
even have already completed but be enqueued within the kernel, this
call may not succeed and the ADC may still issue a callback with the
sample via the `Client` trait. This function MUST return SUCCESS, ERESERVE
or FAIL. SUCCESS indicates that a callback WILL NOT be issued, while
a failure code (FAIL or ERESERVE) indicate that the callback WILL be
issued normally.

3. AdcContinuous
-------------------------------

The AdcContinuous trait is for requesting a stream of ADC conversions
at a fixed frequency. These samples are expected to be taken with
low jitter (e.g., clocked by an underyling hardware clock and not
controlled by software).

Because the AdcContinuous trait takes an interval between samples,
it needs a specify a time unit for this interval. As platforms can
vary by orders of magnitude in the interval they can support, the
trait instance should specify the time unit that represents the
highest possible precision. For example, if an ADC can sample at
maximum rate of 10kHz, the time unit should be 10kHz or less.
Software trying to sample at a significantly higher precision
(e.g., 1MHz ticks) should not be able to compile.

The `Frequency` trait specifies the time unit of continuous
sampling precision.

    pub trait Frequency {
        fn frequency() -> u32;
    }

    pub trait AdcContinuous {
        type Frequency: Frequency;
        fn compute_interval(&self, interval:u32) -> u32;
        fn sample_continuous(&self, channel: u8, interval:u32) -> Result;
        fn cancel_sampling(&self) -> Result;
    }

The AdcContinuous trait has three functions. The first,
`compute_interval` takes a specified interval and returns
the actual interval that the ADC will provide. Whenever
possible, `compute_interval` SHOULD be the identity function.
However, underlying clock rates and hardware details MAY cause
it to be a slightly different value.

The second, `sample_continuous`, starts a series of continuous
samples on `channel` with an interval of `interval`. The
ADC might not sample at exactly the interval specified, due
to underlying clock rates and hardware details. The value
passed in `interval` is effectively passed to `compute_interval`
to calculate the actual interval used. If a call to `sample_continuous`
is passed an `interval` generated by calling `compute_interval`,
then the implementation SHOULD match this interval exactly.

The third, `cancel_sampling`, stops the continuous sampling.

The `Frequency` type defines the frequency of the intervals. There
are three default values of `Frequency`:
  * 1kHz
  * 32kHz
  * 1MHz

4. Client
-------------------------------

The Client trait is how a caller provides a callback to the ADC
implementation. Using a function defined outside the ADC trait, it
registers a reference implementing the Client trait with the ADC
implementation.

    pub trait Client {
        /// Called when a sample is ready.
        fn sample_done(&self, sample: u16, result: Result);
    }

Whenever the ADC completes a sample, it invokes the `sample_done`
function. If the sample was taken successfully, then `sample` MUST
contain the value and `result` MUST be SUCCESS. If `sample` contains a
value, the value MUST be left shifted so the most significant bit of
`sample` is the most significant bit of the value. Possible values for
`result` are SUCCESS, FAIL, EBUSY, EOFF, ERESERVE, EINVAL, or ECANCEL.


5. Example Implementation
---------------------------------

6. Authors' Address
---------------------------------

Philip Levis
409 Gates Hall
Stanford University
Stanford, CA 94305

phone - +1 650 725 9046

email - pal@cs.stanford.edu

7. Citations
---------------------------------

[TRD1] <trd1-trds.md>
