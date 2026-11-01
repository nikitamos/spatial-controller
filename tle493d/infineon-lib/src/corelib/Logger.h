#ifndef TLX493D_LOGGER_H
#define TLX493D_LOGGER_H

#include <stdint.h>
#ifdef __cplusplus

/** std includes. */
#ifdef __AVR__

#include <stdarg.h>

#else

#include <cstdarg>

#endif

extern "C" {

#else

/** std includes. */
#include <stdarg.h>

/** project c includes. */
#include "tlx493d_types.h"

#endif

#if defined(__AVR__) && !defined(ARDUINO_UNOR4_MINIMA) &&                      \
    !defined(ARDUINO_AVR_MEGA2560)

#define logInfo(...) ((void)0)
#define logWarn(...) ((void)0)
#define logError(...) ((void)0)

// #warning "C Preprocessor got here!"

#else

#define logInfo(x) logPrintln(LEVEL_INFO, (x))
#define logWarn(x) logPrintln(LEVEL_WARN, (x))
#define logError(x) logPrint(LEVEL_ERROR, (x))

#endif

/**
 * @brief The function `logPrintRegisters` prints out all the internal registers
 * of the passed sensor object.
 *
 * @param[in] sensor A pointer to a TLx493D_t structure, which represents the
 * TLx493D sensor.
 * @param[in] headLine A string with register names to be printed above the
 * registers for easier reading.
 */
void logPrintRegisters(const TLx493D_t *sensor, const char *headLine);

#define TLX_DECLARE_LOGGER(logger_name, ...)                                   \
  void logger_name(uint8_t level, const char *format, __VA_ARGS__);

TLX_DECLARE_LOGGER(logPrints, const char *)
TLX_DECLARE_LOGGER(logPrintd, int32_t)
TLX_DECLARE_LOGGER(logPrintss, const char *, const char *)
TLX_DECLARE_LOGGER(logPrintds, int32_t, const char *)
TLX_DECLARE_LOGGER(logPrintdss, int32_t, const char *, const char *)
TLX_DECLARE_LOGGER(logPrintdd, int32_t, int32_t)

enum LogLevel { LEVEL_INFO = 0, LEVEL_WARN = 1, LEVEL_ERROR = 2 };

#undef TLX_DECLARE_LOGGER

/**
 * @brief The function `logPrintDouble` prints out a value in the real format.
 *
 * @param[in] d Value which will be printed to the serial monitor.
 */
void logPrintDouble(real d);

void logPrint(uint8_t level, const char *format);

/**
 * @brief The function `logPrintln` is used to print out a formatted string -
 * without prefix - to the serial output plus a trailing linefeed.
 *
 * @param[in] format Actual string, which should be printed to the serial
 * output.
 */
void logPrintln(const char *prefix, const char *format, ...);

/**
 * @brief The function `flush` is used to send a new line character to ther
 * serial output.
 *
 */
void logFlush(void);

#ifdef __cplusplus
}

#endif

#endif /** TLX493D_LOGGER_H */
