#ifndef TLX493D_ARDUINO_BASETYPES_H
#define TLX493D_ARDUINO_BASETYPES_H


/** project cpp includes. */
#include "corelib/Kit2GoBoardSupport.hpp"
#include "SPIClassWrapper.hpp"
#include "TwoWireWrapper.hpp"

#ifdef __cplusplus
extern "C" {
#endif
//

struct RustI2cComm;
struct RustSpiComm;

/**
 * @brief Structure to hold the I2C-object, needed for communication.
 */
using TLx493D_I2CObject_t = RustI2cComm;

/**
 * @brief Structure to hold the SPI-object, needed for communication.
 */
using TLx493D_SPIObject_t = void;

/**
 * @brief Structure to hold the Kit2GoBoardSupport-Object.
 */
using TLx493D_Kit2GoBoardSupportObject_t =
    struct TLx493D_Kit2GoBoardSupportObject {
  char x;
};

#ifdef __cplusplus
} // extern "C"
#endif

#endif // TLX493D_ARDUINO_BASETYPES_H
