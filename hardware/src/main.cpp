#include "main.h"

#include <Arduino.h>
#include <TinyGsmClient.h>
#include <SSLClient.h>
#include <ArduinoHttpClient.h>

#include <HttpWrapper.h>
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
GsmWrapper gsmWrapper(modem, APN, APN_USER, APN_PASSWORD, GSM_MAX_RETRIES);

#if defined(TINY_GSM_MODEM_SIM808)
GpsGsmClient gpsClient(modem);
#else
TinyGPSPlus gps;
GpsClient gpsClient(gps);
#endif // TINY_GSM_MODEM_SIM808

TinyGsmClient client(modem);

#ifdef USE_SSL
SSLClient sslClient(&client);
// Assuming you have define or cert logic
HttpWrapper httpClient(sslClient, TARGET_HOST, TARGET_PORT);
#else
HttpWrapper httpClient(client, TARGET_HOST, TARGET_PORT);
#endif

bool modemConnected = false;
bool systemReady = false;
unsigned short retryCount = 0;
unsigned long lastPublish = 0;
unsigned long sequence = 1;
unsigned long iteration = 1;

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
    Serial.printf("APN          : %s\n", APN);
    Serial.printf("APN User     : %s\n", APN_USER);
    Serial.printf("APN Password : %s\n", APN_PASSWORD);
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

#ifdef USE_SSL
#ifdef USE_INSECURE_SSL
    sslClient.setInsecure();
#else
    sslClient.setCACert(SSL_CA_CERTIFICATE);
#endif
    Serial.println("SSL client initialized!");
#endif


    if (!gsmWrapper.begin())
    {
        Serial.println("Failed to initialize GSM/GPRS modem!");
        return;
    }

#if defined(TINY_GSM_MODEM_SIM808)
    if (!gsmWrapper.enableGPS())
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
        if (!gsmWrapper.begin())
        {
            Serial.println("Failed to initialize GSM/GPRS modem!");
            return;
        }
        
#if defined(TINY_GSM_MODEM_SIM808)
        if (!gsmWrapper.enableGPS())
        {
            Serial.println("Failed to initialize GPS modem!");
            return;
        }
#endif // TINY_GSM_MODEM_SIM808

        systemReady = true;
    }

    if (!gsmWrapper.ensureConnection())
    {
        Serial.println("GPRS connection lost!");
        delay(1000);
        return;
    }

#if defined(TINY_GSM_MODEM_SIM900)
    while (gpsSerial.available() > 0)
    {
        char c = gpsSerial.read();
        gpsClient.encode(c);
    }
#endif // TINY_GSM_MODEM_SIM900

    if (millis() - lastPublish > PUBLISH_INTERVAL)
    {
        JsonDocument payload = gpsClient.toJsonDocument();
        payload["connection"]["interval"] = millis() - lastPublish;
        payload["connection"]["retries"] = retryCount;
        payload["connection"]["sequence_id"] = sequence;
        payload["connection"]["iteration_id"] = iteration;
        payload["connection"]["strength"] = gsmWrapper.getSignalStrength();
        payload["id"] = TRACKER_ID;

        const bool isValid = payload["location"]["valid"];
        String payloadString;
        serializeJson(payload, payloadString);

        isValid
            ? Serial.println("Publishing payload: " + payloadString)
            : Serial.println("Publishing unretained payload: " + payloadString);

        const bool isSuccess = httpClient.post(TARGET_PATH, "application/json", payloadString.c_str());
        if (isSuccess)
        {
            lastPublish = millis();
            retryCount = 0;
            sequence = sequence + 1;
            iteration = iteration + 1;
        }
        else
        {
            retryCount = retryCount + 1;
            iteration = iteration + 1;
        }

        // isSuccess
        //     ? Serial.println("Message published successfully!")
        //     : Serial.println("Failed to publish message!");
    }
}
