#include "main.h"

#include <Arduino.h>
#include <TinyGsmClient.h>
#include <PubSubClient.h>
#include <SSLClient.h>

#include <MqttClient.h>
#include <GsmWrapper.h>

#if defined(TINY_GSM_MODEM_SIM808)
#include <GpsGsmClient.h>
#else
#include <TinyGPS++.h>
#include <GpsClient.h>
#endif // TINY_GSM_MODEM_SIM808

HardwareSerial gsmSerial(2);
HardwareSerial gpsSerial(1);

#ifdef DUMP_AT_COMMANDS
#include <StreamDebugger.h>
StreamDebugger debugger(gsmSerial, Serial);
TinyGsm modem(debugger);
#else
TinyGsm modem(gsmSerial);
#endif

#if defined(TINY_GSM_MODEM_SIM808)
GpsGsmClient gpsClient(modem);
#else
TinyGPSPlus gps;
GpsClient gpsClient(gps);
#endif // TINY_GSM_MODEM_SIM808

bool modemConnected = false;
bool systemReady = false;

void setup()
{
    Serial.begin(115200);
    while (!Serial)
    {
        delay(10);
    }
    Serial.println("Serial initialized!");

    Serial.println("ENVIRONMENT");
    Serial.println("-----------");
    Serial.printf("Device ID    : %d\n", TRACKER_ID);
    Serial.println("-----------");

#if (defined(TINY_GSM_AUTOBAUD))
    {
        gsmSerial.begin(SERIAL_8N1, ESP32_RX_PIN, ESP32_TX_PIN);
        TinyGsmAutoBaud(gsmSerial, TINY_GSM_AUTOBAUD_MINIMUM, TINY_GSM_AUTOBAUD_MAXIMUM);
    }
#else
    {
        gsmSerial.begin(9600, SERIAL_8N1, GSM_RX_PIN, GSM_TX_PIN);
        while (!gsmSerial)
        {
            delay(10);
        }
    }
#endif
    Serial.println("GSM Serial initialized!");

    gpsSerial.begin(9600, SERIAL_8N1, GPS_RX_PIN, GPS_TX_PIN);
    while (!gpsSerial)
    {
        delay(10);
    }
    Serial.println("GPS Serial initialized!");



#if defined(TINY_GSM_MODEM_SIM808)
    if (!gpsClient.enableGps())
    {
        Serial.println("Failed to initialize GPS modem!");
        return;
    }
#endif // TINY_GSM_MODEM_SIM808

    systemReady = true;
}

void loop()
{
    if (!systemReady)
    {
        delay(5000);

#if defined(TINY_GSM_MODEM_SIM808)
        if (!gpsClient.enableGps())
        {
            Serial.println("Failed to initialize GPS modem!");
            return;
        }
#endif // TINY_GSM_MODEM_SIM808

        systemReady = true;
    }
    
#if defined(TINY_GSM_MODEM_SIM808)
    Serial.println("Reading GPS data...");
    String gpsData = gpsClient.getRawGpsData();
    if (gpsData.length() > 0)
    {
        Serial.print("GPS Data:");
        Serial.println(gpsData);    
    }
    else
    {
        Serial.println("No GPS data available.");
    }
#endif // TINY_GSM_MODEM_SIM808

    delay(1000);
}
