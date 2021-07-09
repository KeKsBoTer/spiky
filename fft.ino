/* very experimental implementation of fast fourier transformation on the IMU data */
#include <Adafruit_ZeroFFT.h>
#include <arm_common_tables.h>

#include <Arduino_LSM6DS3.h>

#define SAMPLES 32             //Must be a power of 2
#define SAMPLING_FREQUENCY 104 //Hz, must be less than 10000 due to ADC

unsigned int sampling_period_us;
unsigned long microseconds;

int16_t data[SAMPLES];

void setup()
{
    Serial.begin(9600);
    IMU.begin();

    sampling_period_us = round(1000000 * (1.0 / SAMPLING_FREQUENCY));
}

#define NUM_BINS 15
const float BIN_FREQ[NUM_BINS] = {6.5, 9.75, 13., 16.25, 19.5, 22.75, 26.,
                                  29.25, 32.5, 35.75, 39., 42.25, 45.5, 48.75, 52.};

void loop()
{
    float x, y, z;

    /*SAMPLING*/
    for (int i = 0; i < SAMPLES; i++)
    {
        microseconds = micros(); //Overflows after around 70 minutes!

        IMU.readAcceleration(x, y, z);
        data[i] = int16_t(x * 1000);

        while (micros() < (microseconds + sampling_period_us))
            ;
    }

    // TODO: Subtract mean from data to avoid high values in low frequency area.
    //       Implement overlapping windows for lower latency
    //       Find a way to make sure this Implementation matches the numpy one

    ZeroFFT(data, SAMPLES);

    for (int i = 0; i < NUM_BINS; i++)
    {
        int idx = FFT_INDEX(BIN_FREQ[i], SAMPLING_FREQUENCY, SAMPLES);
        Serial.println(data[idx], 1);
    }
}