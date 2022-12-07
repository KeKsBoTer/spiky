
#include <Arduino.h>
#include <SERCOM.h>
#include <wiring_private.h>
//#include <Arduino_LSM6DS3.h>

#define LSM6DS3_ADDRESS            0x6A

#define LSM6DS3_WHO_AM_I_REG 0X0F


#define PIN_WIRE_SDA         (18u)
#define PIN_WIRE_SCL         (19u)

bool transmissionBegun = false;
uint8_t txAddress;
RingBufferN<256> txBuffer;
RingBufferN<256> rxBuffer;

void Wire_begin(){
  sercom4.initMasterWIRE(100000);
  sercom4.enableWIRE();

  pinPeripheral(PIN_WIRE_SDA, g_APinDescription[PIN_WIRE_SDA].ulPinType);
  pinPeripheral(PIN_WIRE_SCL, g_APinDescription[PIN_WIRE_SCL].ulPinType);
}


void Wire_beginTransmission(uint8_t address) {
  // save address of target and clear buffer
  txAddress = address;
  txBuffer.clear();

  transmissionBegun = true;
}


int error = -1;

// Errors:
//  0 : Success
//  1 : Data too long
//  2 : NACK on transmit of address
//  3 : NACK on transmit of data
//  4 : Other error
uint8_t Wire_endTransmission(bool stopBit)
{
  transmissionBegun = false ;

  // Start I2C transmission
  if ( !sercom4.startTransmissionWIRE( txAddress, WIRE_WRITE_FLAG ) )
  {
    sercom4.prepareCommandBitsWire(WIRE_MASTER_ACT_STOP);
    return 2 ;  // Address error
  }

  // Send all buffer
  while( txBuffer.available() )
  {
    // Trying to send data
    if ( !sercom4.sendDataMasterWIRE( txBuffer.read_char() ) )
    {
      sercom4.prepareCommandBitsWire(WIRE_MASTER_ACT_STOP);
      return 3 ;  // Nack or error
    }
  }
  
  if (stopBit)
  {
    sercom4.prepareCommandBitsWire(WIRE_MASTER_ACT_STOP);
  }   

  return 0;
}

uint8_t Wire_endTransmission()
{
  return Wire_endTransmission(true);
}


size_t Wire_write(uint8_t ucData)
{
  // No writing, without begun transmission or a full buffer
  if ( !transmissionBegun || txBuffer.isFull() )
  {
    return 0 ;
  }

  txBuffer.store_char( ucData ) ;

  return 1 ;
}

size_t Wire_write(const uint8_t *data, size_t quantity)
{
  //Try to store all data
  for(size_t i = 0; i < quantity; ++i)
  {
    //Return the number of data stored, when the buffer is full (if write return 0)
    if(!Wire_write(data[i]))
      return i;
  }

  //All data stored
  return quantity;
}


size_t Wire_requestFrom(uint8_t address, size_t quantity, bool stopBit)
{
  if(quantity == 0)
  {
    return 0;
  }

  size_t byteRead = 0;

  rxBuffer.clear();

  if(sercom4.startTransmissionWIRE(address, WIRE_READ_FLAG))
  {
    // Read first data
    rxBuffer.store_char(sercom4.readDataWIRE());

    bool busOwner;
    // Connected to slave
    for (byteRead = 1; byteRead < quantity && (busOwner = sercom4.isBusOwnerWIRE()); ++byteRead)
    {
      sercom4.prepareAckBitWIRE();                          // Prepare Acknowledge
      sercom4.prepareCommandBitsWire(WIRE_MASTER_ACT_READ); // Prepare the ACK command for the slave
      rxBuffer.store_char(sercom4.readDataWIRE());          // Read data and send the ACK
    }
    sercom4.prepareNackBitWIRE();                           // Prepare NACK to stop slave transmission
    //sercom->readDataWIRE();                               // Clear data register to send NACK

    if (stopBit && busOwner)
    {
      sercom4.prepareCommandBitsWire(WIRE_MASTER_ACT_STOP);   // Send Stop unless arbitration was lost
    }

    if (!busOwner)
    {
      byteRead--;   // because last read byte was garbage/invalid
    }
  }

  return byteRead;
}

size_t Wire_requestFrom(uint8_t address, size_t quantity)
{
  return Wire_requestFrom(address, quantity, true);
}


void Wire_onService(void)
{
  if ( sercom4.isSlaveWIRE() )
  {
    if(sercom4.isStopDetectedWIRE() || 
        (sercom4.isAddressMatch() && sercom4.isRestartDetectedWIRE() && !sercom4.isMasterReadOperationWIRE())) //Stop or Restart detected
    {
      sercom4.prepareAckBitWIRE();
      sercom4.prepareCommandBitsWire(0x03);
      
      rxBuffer.clear();
    }
    else if(sercom4.isAddressMatch())  //Address Match
    {
      sercom4.prepareAckBitWIRE();
      sercom4.prepareCommandBitsWire(0x03);

      if(sercom4.isMasterReadOperationWIRE()) //Is a request ?
      {
        txBuffer.clear();

        transmissionBegun = true;
      }
    }
    else if(sercom4.isDataReadyWIRE())
    {
      if (sercom4.isMasterReadOperationWIRE())
      {
        uint8_t c = 0xff;

        if( txBuffer.available() ) {
          c = txBuffer.read_char();
        }

        transmissionBegun = sercom4.sendDataSlaveWIRE(c);
      } else { //Received data
        if (rxBuffer.isFull()) {
          sercom4.prepareNackBitWIRE(); 
        } else {
          //Store data
          rxBuffer.store_char(sercom4.readDataWIRE());

          sercom4.prepareAckBitWIRE(); 
        }

        sercom4.prepareCommandBitsWire(0x03);
      }
    }
  }
}
void SERCOM4_Handler(void) {
  error = 12;
  Wire_onService();
}


int readRegister(uint8_t address)
{
  uint8_t value;
  
  if (readRegisters(address, &value, sizeof(value)) != 1) {
    error = 12;
    return -1;
  }
  
  return value;
}


int Wire_read(void)
{
  return rxBuffer.read_char();
}


int readRegisters(uint8_t address, uint8_t* data, size_t length)
{
    Wire_beginTransmission(LSM6DS3_ADDRESS);
    if(Wire_write(address) !=1){
      error = 15;
      return 0;
     }

    int code = Wire_endTransmission(false);
    if (code != 0) {
      error = code;
      return -1;
    }

    if (Wire_requestFrom(LSM6DS3_ADDRESS, length) != length) {
      return 0;
    }

    for (size_t i = 0; i < length; i++) {
      *data++ = Wire_read();
    }
    return 1;
}

uint8_t reg_value =0;

void setup() {
  Wire_begin();
  reg_value = readRegister(LSM6DS3_WHO_AM_I_REG);
//  
//  if (!(readRegister(LSM6DS3_WHO_AM_I_REG) == 0x6C || readRegister(LSM6DS3_WHO_AM_I_REG) == 0x69)) {
//    error = 2;
//  }
//
//  if( IMU.begin()==0){
//    reg_value = 3;
//    
//  }
}
void loop() {

  Serial.print(reg_value,BIN);
  Serial.print('\t');
  Serial.print(error,DEC);
  Serial.print('\n');
  delay(1000);

}
