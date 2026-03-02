#include <../../include/main.h>
#include "GsmWrapper.h"

GsmWrapper::GsmWrapper(
    TinyGsm &gsmModem,
    const char *apn,
    const char *apnUser,
    const char *apnPassword,
    const unsigned short maxRetries,
    const unsigned long connectionCheckInterval)
    : gsmModem(gsmModem), apn(apn), apnUser(apnUser), apnPassword(apnPassword), maxRetries(maxRetries), connectionCheckInterval(connectionCheckInterval), lastConnectionCheck(0), connected(false)
{
#if defined(TINY_GSM_MODEM_SIM808)
    this->gps = false;
#endif // TINY_GSM_MODEM_SIM808
}

GsmWrapper::~GsmWrapper() = default;

bool GsmWrapper::initializeModem()
{
    if (!gsmModem.init())
    {
        Serial.println("Could not initialize GSM modem!");
        return false;
    }
    Serial.println("GSM modem initialized!");
    String modemInfo = gsmModem.getModemInfo();
    Serial.println("Using modem: " + modemInfo);
    return true;
}

bool GsmWrapper::connectGprs()
{
    if (this->gsmModem.isGprsConnected())
    {
        Serial.println("GPRS already connected!");
        return true;
    }

    Serial.println("Connecting to GPRS...");
    if (this->gsmModem.gprsConnect(apn, apnUser, apnPassword))
    {
        Serial.println("GPRS connected!");
        return true;
    }

    Serial.println("Failed to connect GPRS!");
    return false;
}

bool GsmWrapper::begin()
{
    unsigned short retries = 0;

    while (retries < this->maxRetries)
    {
        if (!this->initializeModem())
        {
            Serial.println("Retry initializing modem...");
            retries++;
            delay(2000);
            continue;
        }

        if (!connectGprs())
        {
            Serial.println("Retry connecting to GPRS...");
            // Check later on!
            this->gsmModem.gprsDisconnect();
            retries++;
            delay(2000);
            continue;
        }

        String ipAddress = this->getLocalIP();
        Serial.println("Using IP address: " + ipAddress);

        connected = true;
        lastConnectionCheck = millis();
        return true;
    }

    Serial.println("Failed to connect to GPRS!");
    connected = false;
    return false;
}

bool GsmWrapper::isConnected()
{
    return connected && this->gsmModem.isGprsConnected();
}

bool GsmWrapper::ensureConnection()
{
    // Don't reconnect when not connected if below `connectionCheckInterval`
    if (millis() - this->lastConnectionCheck < this->connectionCheckInterval)
    {
        return this->isConnected();
    }

    lastConnectionCheck = millis();

    // Reconnect when not connected
    if (!this->isConnected())
    {
        Serial.println("Reconnecting to GPRS...");
        if (connectGprs())
        {
            connected = true;
            return true;
        }
        connected = false;
        return false;
    }

    if (this->getSignalStrength() < 10)
    {
        Serial.println("Warning: Low signal quality");
    }

    return true;
}

int GsmWrapper::getSignalStrength()
{
    return this->gsmModem.getSignalQuality();
}

String GsmWrapper::getLocalIP()
{
    return this->gsmModem.getLocalIP();
}

void GsmWrapper::disconnect()
{
    if (connected)
    {
        this->gsmModem.gprsDisconnect();
        connected = false;
    }
}

#if defined(TINY_GSM_MODEM_SIM808)
bool GsmWrapper::enableGPS()
{
    const bool success = this->gsmModem.enableGPS();
    if (success)
    {
        this->gps = true;
    }
    return success;
}

bool GsmWrapper::disableGPS()
{
    const bool success = this->gsmModem.disableGPS();
    if (success)
    {
        this->gps = false;
    }
    return success;
}
bool GsmWrapper::statusGPS()
{
    return this->gps;
}
String GsmWrapper::getRawGPS()
{
    return this->gsmModem.getGPSraw();
}
#endif // TINY_GSM_MODEM_SIM808