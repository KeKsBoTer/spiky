/* This code runs on an Arduino Nano 33 IOT or similar devices.
It reads the acceleration values of the inbuilt IMU and sends them to a connected device via BLE.
*/

#include <Arduino_LSM6DS3.h>
#include <ArduinoBLE.h>

BLEService imuService("00002a24-0000-1000-8000-00805f9b34fb");

// The arduino sends 7 values: 3 acceleration values, 3 orientation values and one timestamp (relative to start)
BLECharacteristic vecFloat("2A01", BLERead | BLENotify, 7 * sizeof(float));

void setup()
{
    Serial.begin(9600);
    IMU.begin();

    pinMode(LED_BUILTIN, OUTPUT);
    if (!BLE.begin())
    {
        Serial.println("starting BLE failed!");
        while (1)
            ;
    }

    BLE.setLocalName("IMU Monitor");
    BLE.setDeviceName("IMU Monitor");
    BLE.setAdvertisedService(imuService);
    imuService.addCharacteristic(vecFloat);
    BLE.addService(imuService);

    BLE.advertise();
    Serial.println("Bluetooth device active, waiting for connections...");
}

void loop()
{
    float vec[7];
    BLEDevice central = BLE.central();

    if (central)
    {
        Serial.print("Connected to central: ");
        Serial.println(central.address());
        digitalWrite(LED_BUILTIN, HIGH);

        while (central.connected())
        {
            vec[6] = float(millis());
            IMU.readAcceleration(vec[0], vec[1], vec[2]);
            IMU.readGyroscope(vec[3], vec[4], vec[5]);
            vecFloat.writeValue((byte *)vec, sizeof(vec), false);
            central.poll();
        }
        digitalWrite(LED_BUILTIN, LOW);
    }
}