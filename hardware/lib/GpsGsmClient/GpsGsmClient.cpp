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
    String rawGPS = this->gsmModem.getGPSraw();

    // If the raw string hasn't changed, retry up to 5 times (1 s each)
    // to wait for a fresh GPS fix.
    if (rawGPS == this->_lastRawGPS) {
        for (int attempt = 0; attempt < 5; attempt++) {
            delay(1000);
            rawGPS = this->gsmModem.getGPSraw();
            if (rawGPS != this->_lastRawGPS) {
                break;
            }
        }
    }
    this->_lastRawGPS = rawGPS;
    char buffer[128];
    rawGPS.toCharArray(buffer, sizeof(buffer));
    
    int index = 0;
    char* tokenStart = buffer;
    
    int gnssRunStatus = 0;  // Index 0
    bool fixStatus = 0;     // Index 1
    String iso8601 = "";    // Index 2
    double latitude = 0;    // Index 3
    double longitude = 0;   // Index 4
    float altitude = 0;     // Index 5
    float speed = 0;        // Index 6
    float course = 0;       // Index 7
    float HDOP = 0;         // Index 10
    float PDOP = 0;         // Index 11
    float VDOP = 0;         // Index 12
    int vsat = 0;           // Index 14
    int usat = 0;           // Index 15
    int carrierToNoise = 0; // Index 18
    int year = 0, month = 0, day = 0, hour = 0, minute = 0, second = 0;
    bool timeValid = false;

    while (tokenStart != nullptr) {
        char* tokenEnd = strchr(tokenStart, ',');
        if (tokenEnd != nullptr) {
            *tokenEnd = '\0';
        }

        if (tokenStart[0] != '\0') {
            switch (index) {
                case 0: gnssRunStatus = atoi(tokenStart); break;
                case 1: fixStatus = atoi(tokenStart) == 1; break;
                case 2: if (sscanf(tokenStart, "%4d%2d%2d%2d%2d%2d", &year, &month, &day, &hour, &minute, &second) == 6) {
                        timeValid = true;
                    }
                    break;
                case 3: latitude = atof(tokenStart); break;
                case 4: longitude = atof(tokenStart); break;
                case 5: altitude = atof(tokenStart); break;
                case 6: speed = atof(tokenStart); break;
                case 7: course = atof(tokenStart); break;
                case 10: HDOP = atof(tokenStart); break;
                case 11: PDOP = atof(tokenStart); break;
                case 12: VDOP = atof(tokenStart); break;
                case 14: vsat = atoi(tokenStart); break;
                case 15: usat = atoi(tokenStart); break;
                case 18: carrierToNoise = atoi(tokenStart); break;
            }
        }

        if (tokenEnd != nullptr) {
            tokenStart = tokenEnd + 1;
        } else {
            tokenStart = nullptr;
        }
        index++;
    }

    // Location
    if (latitude == 0.0 && longitude == 0.0) {
        this->jsonDoc["location"]["latitude"] = nullptr;
        this->jsonDoc["location"]["longitude"] = nullptr;
        this->jsonDoc["location"]["age"] = nullptr;
        this->jsonDoc["location"]["valid"] = false;
    } else {
        this->jsonDoc["location"]["latitude"] = latitude;
        this->jsonDoc["location"]["longitude"] = longitude;
        this->jsonDoc["location"]["age"] = nullptr;
        this->jsonDoc["location"]["valid"] = true;
    }
    
    // Altitude
    if (altitude == 0.0) {
        this->jsonDoc["altitude"]["meters"] = nullptr;
        this->jsonDoc["altitude"]["feet"] = nullptr;
        this->jsonDoc["altitude"]["age"] = nullptr;
        this->jsonDoc["altitude"]["valid"] = false;
    } else {
        this->jsonDoc["altitude"]["meters"] = altitude;
        this->jsonDoc["altitude"]["feet"] = this->feet(altitude);
        this->jsonDoc["altitude"]["age"] = nullptr;
        this->jsonDoc["altitude"]["valid"] = true;
    }

    // Speed
    if (speed == 0.0) {
        this->jsonDoc["speed"]["kmph"] = nullptr;
        this->jsonDoc["speed"]["mph"] = nullptr;
        this->jsonDoc["speed"]["mps"] = nullptr;
        this->jsonDoc["speed"]["knots"] = nullptr;
        this->jsonDoc["speed"]["age"] = nullptr;
        this->jsonDoc["speed"]["valid"] = false;
    } else {
        this->jsonDoc["speed"]["kmph"] = this->kmph(speed);
        this->jsonDoc["speed"]["mph"] = this->mph(speed);
        this->jsonDoc["speed"]["mps"] = this->mps(speed);
        this->jsonDoc["speed"]["knots"] = speed;
        this->jsonDoc["speed"]["age"] = nullptr;
        this->jsonDoc["speed"]["valid"] = true;
    }
    
    // Date and Time
    if (timeValid) {
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
        this->jsonDoc["datetime"]["valid"] = (year > 2000);
    } else {
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
    }
    
    // Course
    if (course == 0.0) {
        this->jsonDoc["course"]["degrees"] = nullptr;
        this->jsonDoc["course"]["age"] = nullptr;
        this->jsonDoc["course"]["valid"] = false;
    } else {
        this->jsonDoc["course"]["degrees"] = course;
        this->jsonDoc["course"]["age"] = nullptr;
        this->jsonDoc["course"]["valid"] = true;
    }
    
    // DOP
    if (HDOP == 0.0 && PDOP == 0.0 && VDOP == 0.0) {
        this->jsonDoc["dop"]["hdop"] = nullptr;
        this->jsonDoc["dop"]["pdop"] = nullptr;
        this->jsonDoc["dop"]["vdop"] = nullptr;
        this->jsonDoc["dop"]["age"] = nullptr;
        this->jsonDoc["dop"]["valid"] = false;
    } else {
        this->jsonDoc["dop"]["hdop"] = HDOP;
        this->jsonDoc["dop"]["pdop"] = PDOP;
        this->jsonDoc["dop"]["vdop"] = VDOP;
        this->jsonDoc["dop"]["age"] = nullptr;
        this->jsonDoc["dop"]["valid"] = true;
    }
    
    // Satellites
    this->jsonDoc["satellites"]["visible"] = vsat;
    this->jsonDoc["satellites"]["used"] = usat;
    this->jsonDoc["satellites"]["carrier_to_noise"] = carrierToNoise;
    this->jsonDoc["satellites"]["age"] = nullptr;
    this->jsonDoc["satellites"]["valid"] = true;

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