#include "GpsGsmClient.h"
#if defined(TINY_GSM_MODEM_SIM808)

GpsGsmClient::GpsGsmClient(TinyGsm &gsmModem) : gsmModem(gsmModem)
{
}

GpsGsmClient::~GpsGsmClient() = default;

JsonDocument &GpsGsmClient::toJsonDocument()
{
    // Clear per-instance document
    this->jsonDoc.clear();

    // Add timestamp
    this->jsonDoc["uptime"] = millis();

    // Location Data
    float latitude = 0;
    float longitude = 0;
    float speed = 0;    // In knots
    float altitude = 0; // In meters
    float accuracy = 0;
    int vsat = 0;
    int usat = 0;
    int year = 0;
    int month = 0;
    int day = 0;
    int hour = 0;
    int minute = 0;
    int second = 0;

    bool hasGPSFix = this->gsmModem.getGPS(&latitude, &longitude, &speed, &altitude, &vsat, &usat, &accuracy, &year, &month, &day, &hour, &minute, &second);

    if (hasGPSFix)
    {
        // Location
        this->jsonDoc["location"]["latitude"] = latitude;
        this->jsonDoc["location"]["longitude"] = longitude;
        this->jsonDoc["location"]["age"] = nullptr;
        this->jsonDoc["location"]["valid"] = true;

        // Altitude
        this->jsonDoc["altitude"]["meters"] = this->meters(altitude);
        this->jsonDoc["altitude"]["feet"] = this->feet(altitude);
        this->jsonDoc["altitude"]["age"] = nullptr;
        this->jsonDoc["altitude"]["valid"] = true;

        // Speed
        this->jsonDoc["speed"]["kmph"] = this->kmph(speed);
        this->jsonDoc["speed"]["mph"] = this->mph(speed);
        this->jsonDoc["speed"]["mps"] = this->mps(speed);
        this->jsonDoc["speed"]["knots"] = this->knots(speed);
        this->jsonDoc["speed"]["age"] = nullptr;
        this->jsonDoc["speed"]["valid"] = true;

        // Date and Time
        char dateTimeBuffer[32] = "";
        snprintf(dateTimeBuffer, sizeof(dateTimeBuffer),
                 "%04d-%02d-%02dT%02d:%02d:%02dZ",
                 year, month, day, hour, minute, second);
        this->jsonDoc["datetime"]["iso8601"] = dateTimeBuffer;
        this->jsonDoc["datetime"]["year"] = year;
        this->jsonDoc["datetime"]["month"] = month;
        this->jsonDoc["datetime"]["day"] = day;
        this->jsonDoc["datetime"]["hour"] = hour;
        this->jsonDoc["datetime"]["minute"] = minute;
        this->jsonDoc["datetime"]["second"] = second;
        this->jsonDoc["datetime"]["centisecond"] = 0;
        this->jsonDoc["datetime"]["age"] = nullptr;
        this->jsonDoc["datetime"]["valid"] = true;

        this->jsonDoc["satellites"]["count"] = usat;
        this->jsonDoc["satellites"]["age"] = nullptr;
        this->jsonDoc["satellites"]["valid"] = true;
    }
    else
    {
        // Location
        this->jsonDoc["location"]["latitude"] = nullptr;
        this->jsonDoc["location"]["longitude"] = nullptr;
        this->jsonDoc["location"]["age"] = nullptr;
        this->jsonDoc["location"]["valid"] = false;

        // Altitude
        this->jsonDoc["altitude"]["meters"] = nullptr;
        this->jsonDoc["altitude"]["feet"] = nullptr;
        this->jsonDoc["altitude"]["age"] = nullptr;
        this->jsonDoc["altitude"]["valid"] = false;

        // Speed
        this->jsonDoc["speed"]["kmph"] = nullptr;
        this->jsonDoc["speed"]["mph"] = nullptr;
        this->jsonDoc["speed"]["mps"] = nullptr;
        this->jsonDoc["speed"]["knots"] = nullptr;
        this->jsonDoc["speed"]["age"] = nullptr;
        this->jsonDoc["speed"]["valid"] = false;

        // Date and Time
        this->jsonDoc["datetime"]["iso8601"] = nullptr;
        this->jsonDoc["datetime"]["year"] = nullptr;
        this->jsonDoc["datetime"]["month"] = nullptr;
        this->jsonDoc["datetime"]["day"] = nullptr;
        this->jsonDoc["datetime"]["hour"] = nullptr;
        this->jsonDoc["datetime"]["minute"] = nullptr;
        this->jsonDoc["datetime"]["second"] = nullptr;
        this->jsonDoc["datetime"]["centisecond"] = nullptr;
        this->jsonDoc["datetime"]["age"] = nullptr;
        this->jsonDoc["datetime"]["valid"] = false;

        // Satellites
        this->jsonDoc["satellites"]["count"] = nullptr;
        this->jsonDoc["satellites"]["age"] = nullptr;
        this->jsonDoc["satellites"]["valid"] = nullptr;
    }

    // Course/Direction
    this->jsonDoc["course"]["degrees"] = nullptr;
    this->jsonDoc["course"]["age"] = nullptr;
    this->jsonDoc["course"]["valid"] = false;

    // HDOP
    this->jsonDoc["hdop"]["value"] = nullptr;
    this->jsonDoc["hdop"]["age"] = nullptr;
    this->jsonDoc["hdop"]["valid"] = false;

    // Statistics
    this->jsonDoc["stats"]["chars_processed"] = nullptr;
    this->jsonDoc["stats"]["sentences_with_fix"] = nullptr;
    this->jsonDoc["stats"]["failed_checksum"] = nullptr;
    this->jsonDoc["stats"]["passed_checksum"] = nullptr;

    return this->jsonDoc;
}

String GpsGsmClient::toJsonString()
{
    JsonDocument &doc = toJsonDocument();
    String jsonString;
    serializeJson(doc, jsonString);
    return jsonString;
}

String GpsGsmClient::getRawGpsData()
{
    String rawData = this->gsmModem.getGPSraw();
    return rawData;
}

float GpsGsmClient::meters(const float &meters)
{
    return meters;
}
float GpsGsmClient::miles(const float &meters)
{
    return meters * _GPS_MILES_PER_METER;
}
float GpsGsmClient::kilometers(const float &meters)
{
    return meters * _GPS_KM_PER_METER;
}
float GpsGsmClient::feet(const float &meters)
{
    return meters * _GPS_FEET_PER_METER;
}

float GpsGsmClient::knots(const float &knots)
{
    return knots;
}
float GpsGsmClient::mph(const float &knots)
{
    return knots * _GPS_MPH_PER_KNOT;
}
float GpsGsmClient::mps(const float &knots)
{
    return knots * _GPS_MPS_PER_KNOT;
}
float GpsGsmClient::kmph(const float &knots)
{
    return knots * _GPS_KMPH_PER_KNOT;
}
#endif // TINY_GSM_MODEM_SIM808