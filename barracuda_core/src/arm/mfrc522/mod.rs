/*
*   # The MRFC522 Module
*   This module implements reading RFID media (ISO 14443A),
*   using the NXP MRFC522 reader chip
*
*   The code in this module is loosely based on 
*   "Pi My Life Up's guide on setting up an RFID RC522"
*   implementation
*
*   ## Usage
*   
*   ### Configuration
*   The module expects to be provided with a completely configured
*   SPI instance.
*   A this point it only supports reading the UID of ISO 14443A tags.
*
*
*   ### Behavior
*   After the module is started it will search for media every 50 ms
*   and generate door-open requests everytime it sees a medium.
*
*   ### Notes
*   The original code does not use IRQs, but instead uses polling
*   (stupidly!). This should be 
*/

mod mfrc522;