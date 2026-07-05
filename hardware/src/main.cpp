#include "main.h"

#include <Arduino.h>
#include <TinyGsmClient.h>
#include <PubSubClient.h>
#include <SSLClient.h>

#include <MqttClient.h>
#include <GsmWrapper.h>

#include <GpsGsmClient.h>

HardwareSerial gsmSerial(1);

#ifdef DUMP_AT_COMMANDS
#include <StreamDebugger.h>
StreamDebugger debugger(gsmSerial, Serial);
TinyGsm modem(debugger);
#else
TinyGsm modem(gsmSerial);
#endif
GsmWrapper gsmWrapper(modem, APN, APN_USER, APN_PASSWORD, GSM_MAX_RETRIES);

TinyGsmClient client(modem);
GpsGsmClient gpsClient(modem);

#ifdef MQTT_SECURE
SSLClient sslClient(&client);
PubSubClient mqtt(sslClient);
MqttClient mqttClient(mqtt, MQTT_BROKER, MQTT_PORT, MQTT_TOPIC, MQTT_MESSAGE_SIZE, MQTT_KEEP_ALIVE_TIMEOUT, MQTT_RECONNECT_TIMEOUT, MQTT_USERNAME, MQTT_PASSWORD);

#elif defined(MQTT_USERNAME)
PubSubClient mqtt(client);
MqttClient mqttClient(mqtt, MQTT_BROKER, MQTT_PORT, MQTT_TOPIC, MQTT_MESSAGE_SIZE, MQTT_KEEP_ALIVE_TIMEOUT, MQTT_RECONNECT_TIMEOUT, MQTT_USERNAME, MQTT_PASSWORD);

#else
PubSubClient mqtt(client);
MqttClient mqttClient(mqtt, MQTT_BROKER, MQTT_PORT, MQTT_TOPIC, MQTT_MESSAGE_SIZE, MQTT_KEEP_ALIVE_TIMEOUT, MQTT_RECONNECT_TIMEOUT);

#endif

bool modemConnected = false;
bool systemReady = false;
unsigned short retryCount = 0;
unsigned long lastPublish = 0;
unsigned long retryTimeout = 0;
unsigned long sequence = 1;
unsigned long iteration = 1;

static void flashSim808Power();
bool initialization();

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

    pinMode(SIM808_POWER_PIN_PRIMARY, OUTPUT);
    pinMode(SIM808_POWER_PIN_SECONDARY, OUTPUT);
    digitalWrite(SIM808_POWER_PIN_PRIMARY, HIGH);
    digitalWrite(SIM808_POWER_PIN_SECONDARY, HIGH);
    Serial.println("GSM Power Pin Initialized");

#ifdef MQTT_SECURE
    sslClient.setCACert(MQTT_CA_CERTIFICATE);
    Serial.println("SSL client initialized!");
#endif

    if (!initialization())
    {
        Serial.println("Failed to initialize system!");
        return;
    }
}

void loop()
{
    if (!systemReady)
    {
        delay(5000);
        if (!initialization())
        {
            return;
        }
    }

    if (!gsmWrapper.ensureConnection())
    {
        Serial.println("GPRS connection lost!");
        delay(1000);
        return;
    }

    mqttClient.loop();
    if (!mqttClient.isConnected())
    {
        if (!mqttClient.connect())
        {
            delay(1000);
            return;
        }
    }

    if (millis() - retryTimeout > PUBLISH_INTERVAL)
    {
        JsonDocument payload = gpsClient.toJsonDocument();
        payload["connection"]["interval"] = millis() - lastPublish;
        payload["connection"]["retries"] = retryCount;
        payload["connection"]["sequence_id"] = sequence;
        payload["connection"]["iteration_id"] = iteration;
        payload["network"]["rssi"] = gsmWrapper.getSignalStrength();
        payload["id"] = TRACKER_ID;

        String lac, ci;
        if (gsmWrapper.getNetworkInfo(lac, ci))
        {
            payload["network"]["lac"] = lac;
            payload["network"]["ci"] = ci;
        }
        else
        {
            payload["network"]["lac"] = nullptr;
            payload["network"]["ci"] = nullptr;
        }

        const bool isValid = payload["location"]["valid"];
        String payloadString;
        serializeJson(payload, payloadString);

        isValid
            ? Serial.println("Publishing payload: " + payloadString)
            : Serial.println("Publishing unretained payload: " + payloadString);

        const bool isSuccess = mqttClient.publishToDefaultTopic(String(TRACKER_ID).c_str(), payloadString.c_str(), isValid);
        if (isSuccess)
        {
            lastPublish = millis();
            retryTimeout = lastPublish;
            retryCount = 0;
            sequence = sequence + 1;
            iteration = iteration + 1;
        }
        else
        {
            retryTimeout = millis();
            retryCount = retryCount + 1;
            iteration = iteration + 1;
        }

        // isSuccess
        //     ? Serial.println("Message published successfully!")
        //     : Serial.println("Failed to publish message!");
    }
    delay(1000);
}

bool initialization()
{
    if (!gsmWrapper.isActive())
    {
        Serial.println("SIM808 not responsive, flashing power!");
        flashSim808Power();
        if (!gsmWrapper.isActive())
        {
            Serial.println("Failed to activate modem!");
            return false;
        }
    }

    if (!gsmWrapper.begin())
    {
        Serial.println("Failed to initialize GSM/GPRS modem!");
        systemReady = false;
        return false;
    }

    if (!mqttClient.connect())
    {
        Serial.println("Failed to connect to MQTT broker!");
        systemReady = false;
        return false;
    }

    if (!gsmWrapper.enableGPS())
    {
        Serial.println("Failed to initialize GPS modem!");
        systemReady = false;
        return false;
    }

    systemReady = true;
    return true;
}

static void flashSim808Power()
{
    digitalWrite(SIM808_POWER_PIN_PRIMARY, LOW);
    digitalWrite(SIM808_POWER_PIN_SECONDARY, LOW);
    delay(1200);
    digitalWrite(SIM808_POWER_PIN_PRIMARY, HIGH);
    digitalWrite(SIM808_POWER_PIN_SECONDARY, HIGH);
}

