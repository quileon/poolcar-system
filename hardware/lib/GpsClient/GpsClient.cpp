#include "GpsClient.h"

GpsClient::GpsClient(TinyGPSPlus &gps) : gps(gps)
{
}

GpsClient::~GpsClient() = default;

void GpsClient::encode(char c)
{
    gps.encode(c);
}

JsonDocument &GpsClient::toJsonDocument()
{
    // Clear the per-instance document for fresh data
    jsonDoc.clear();

    // Add timestamp
    jsonDoc["uptime"] = millis();

    // Location data
    if (gps.location.isValid())
    {
        jsonDoc["location"]["latitude"] = gps.location.lat();
        jsonDoc["location"]["longitude"] = gps.location.lng();
        jsonDoc["location"]["age"] = gps.location.age();
        jsonDoc["location"]["valid"] = true;
    }
    else
    {
        jsonDoc["location"]["latitude"] = nullptr;
        jsonDoc["location"]["longitude"] = nullptr;
        jsonDoc["location"]["age"] = nullptr;
        jsonDoc["location"]["valid"] = false;
    }

    // Altitude data
    if (gps.altitude.isValid())
    {
        jsonDoc["altitude"]["meters"] = gps.altitude.meters();
        jsonDoc["altitude"]["feet"] = gps.altitude.feet();
        jsonDoc["altitude"]["age"] = gps.altitude.age();
        jsonDoc["altitude"]["valid"] = true;
    }
    else
    {
        jsonDoc["altitude"]["meters"] = nullptr;
        jsonDoc["altitude"]["feet"] = nullptr;
        jsonDoc["altitude"]["age"] = nullptr;
        jsonDoc["altitude"]["valid"] = false;
    }

    // Speed data
    if (gps.speed.isValid())
    {
        jsonDoc["speed"]["kmph"] = gps.speed.kmph();
        jsonDoc["speed"]["mph"] = gps.speed.mph();
        jsonDoc["speed"]["mps"] = gps.speed.mps();
        jsonDoc["speed"]["knots"] = gps.speed.knots();
        jsonDoc["speed"]["age"] = gps.speed.age();
        jsonDoc["speed"]["valid"] = true;
    }
    else
    {
        jsonDoc["speed"]["kmph"] = nullptr;
        jsonDoc["speed"]["mph"] = nullptr;
        jsonDoc["speed"]["mps"] = nullptr;
        jsonDoc["speed"]["knots"] = nullptr;
        jsonDoc["speed"]["age"] = nullptr;
        jsonDoc["speed"]["valid"] = false;
    }

    // Course/Direction data
    if (gps.course.isValid())
    {
        jsonDoc["course"]["degrees"] = gps.course.deg();
        jsonDoc["course"]["age"] = gps.course.age();
        jsonDoc["course"]["valid"] = true;
    }
    else
    {
        jsonDoc["course"]["degrees"] = nullptr;
        jsonDoc["course"]["age"] = nullptr;
        jsonDoc["course"]["valid"] = false;
    }

    // Date and Time data
    if (gps.date.isValid() && gps.time.isValid())
    {
        // Create ISO 8601 formatted date string
        char dateTimeBuffer[32];
        snprintf(dateTimeBuffer, sizeof(dateTimeBuffer),
                 "%04d-%02d-%02dT%02d:%02d:%02dZ",
                 gps.date.year(), gps.date.month(), gps.date.day(),
                 gps.time.hour(), gps.time.minute(), gps.time.second());

        jsonDoc["datetime"]["iso8601"] = dateTimeBuffer;
        jsonDoc["datetime"]["year"] = gps.date.year();
        jsonDoc["datetime"]["month"] = gps.date.month();
        jsonDoc["datetime"]["day"] = gps.date.day();
        jsonDoc["datetime"]["hour"] = gps.time.hour();
        jsonDoc["datetime"]["minute"] = gps.time.minute();
        jsonDoc["datetime"]["second"] = gps.time.second();
        jsonDoc["datetime"]["centisecond"] = gps.time.centisecond();
        jsonDoc["datetime"]["age"] = gps.time.age();
        jsonDoc["datetime"]["valid"] = true;
    }
    else
    {
        jsonDoc["datetime"]["iso8601"] = nullptr;
        jsonDoc["datetime"]["year"] = nullptr;
        jsonDoc["datetime"]["month"] = nullptr;
        jsonDoc["datetime"]["day"] = nullptr;
        jsonDoc["datetime"]["hour"] = nullptr;
        jsonDoc["datetime"]["minute"] = nullptr;
        jsonDoc["datetime"]["second"] = nullptr;
        jsonDoc["datetime"]["centisecond"] = nullptr;
        jsonDoc["datetime"]["age"] = nullptr;
        jsonDoc["datetime"]["valid"] = false;
    }

    // Satellite data
    if (gps.satellites.isValid())
    {
        jsonDoc["satellites"]["count"] = gps.satellites.value();
        jsonDoc["satellites"]["age"] = gps.satellites.age();
        jsonDoc["satellites"]["valid"] = true;
    }
    else
    {
        jsonDoc["satellites"]["count"] = nullptr;
        jsonDoc["satellites"]["age"] = nullptr;
        jsonDoc["satellites"]["valid"] = false;
    }

    // HDOP (Horizontal Dilution of Precision)
    if (gps.hdop.isValid())
    {
        jsonDoc["hdop"]["value"] = gps.hdop.hdop();
        jsonDoc["hdop"]["age"] = gps.hdop.age();
        jsonDoc["hdop"]["valid"] = true;
    }
    else
    {
        jsonDoc["hdop"]["value"] = nullptr;
        jsonDoc["hdop"]["age"] = nullptr;
        jsonDoc["hdop"]["valid"] = false;
    }

    // Statistics
    jsonDoc["stats"]["chars_processed"] = gps.charsProcessed();
    jsonDoc["stats"]["sentences_with_fix"] = gps.sentencesWithFix();
    jsonDoc["stats"]["failed_checksum"] = gps.failedChecksum();
    jsonDoc["stats"]["passed_checksum"] = gps.passedChecksum();

    return jsonDoc;
}

String GpsClient::toJsonString()
{
    JsonDocument &doc = toJsonDocument();
    String jsonString;
    serializeJson(doc, jsonString);
    return jsonString;
}
