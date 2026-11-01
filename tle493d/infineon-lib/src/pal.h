#ifndef TLX493D_PAL_H
#define TLX493D_PAL_H


typedef struct TLx493D_t                           TLx493D_t;

typedef struct TLx493D_I2CObject_t                 TLx493D_I2CObject_t;
typedef struct TLx493D_SPIObject_t                 TLx493D_SPIObject_t;

typedef struct TLx493D_Kit2GoBoardSupportObject_t  TLx493D_Kit2GoBoardSupportObject_t;

#ifdef TLX493D_USE_DOUBLES
typedef double real;
#else
typedef float real;
#endif

/// Rounds argument to the nearest integer value, rounding halfway cases away
/// from zero.
/// This function is equivalent to C99 `lround`.
/// If x is out of range, NaN or infinite, this function causes UB.
long real_round(real x);

#endif // TLX493D_PAL_H
