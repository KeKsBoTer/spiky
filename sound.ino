const int buzzer = 2; //buzzer to arduino pin 9

void setup()
{

    pinMode(buzzer, OUTPUT);     // Set buzzer - pin 9 as an output
    pinMode(PIN_LED_13, OUTPUT); // Set buzzer - pin 9 as an output
}

void loop()
{
    tone(buzzer, 1000);
    digitalWrite(PIN_LED_13, HIGH);
    delay(1000);
    noTone(buzzer);
    digitalWrite(PIN_LED_13, LOW);
    delay(1000);
    // tone(buzzer, 100); // Send 1KHz sound signal...
    // delay(1000);       // ...for 1 sec
    // noTone(buzzer);    // Stop sound...
    // delay(1000);       // ...for 1sec
}
