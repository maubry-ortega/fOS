//! Raspberry Pi Mailbox Interface
//!
//! This module provides functions for communicating with the Raspberry Pi's
//! VideoCore GPU via the mailbox interface. This is necessary to configure
//! and obtain information about the framebuffer.

use core::ptr;

// Mailbox registers (BCM2835 ARM Peripherals manual, Section 1.3.1)
// Mailbox registers (BCM2835 ARM Peripherals manual, Section 1.3.1)
const MAILBOX_BASE: usize = 0x3F00_B880; // Mailbox base address for RPi3
const MAILBOX_READ: *mut u32 = (MAILBOX_BASE + 0x00) as *mut u32;
#[allow(dead_code)]
const MAILBOX_POLL: *mut u32 = (MAILBOX_BASE + 0x10) as *mut u32;
#[allow(dead_code)]
const MAILBOX_SENDER: *mut u32 = (MAILBOX_BASE + 0x14) as *mut u32;
const MAILBOX_STATUS: *mut u32 = (MAILBOX_BASE + 0x18) as *mut u32;
#[allow(dead_code)]
const MAILBOX_CONFIG: *mut u32 = (MAILBOX_BASE + 0x1C) as *mut u32;
const MAILBOX_WRITE: *mut u32 = (MAILBOX_BASE + 0x20) as *mut u32;

// Mailbox status bits
const MAILBOX_FULL: u32 = 0x8000_0000;
const MAILBOX_EMPTY: u32 = 0x4000_0000;

// Mailbox channels
const MAILBOX_CHANNEL_PROPERTY_TAGS_ARM_TO_VC: u32 = 8;

// Property tags for framebuffer
pub const MBOX_TAG_SET_PHYSICAL_DISPLAY_SIZE: u32 = 0x00048003;
pub const MBOX_TAG_SET_VIRTUAL_DISPLAY_SIZE: u32 = 0x00048004;
pub const MBOX_TAG_SET_DEPTH: u32 = 0x00048005;
pub const MBOX_TAG_ALLOCATE_BUFFER: u32 = 0x00040001;
pub const MBOX_TAG_GET_PITCH: u32 = 0x00040008;

// Request and response codes
pub const MBOX_REQUEST: u32 = 0;
const MBOX_RESPONSE_SUCCESS: u32 = 0x8000_0000;
#[allow(dead_code)]
const MBOX_RESPONSE_ERROR: u32 = 0x8000_0001;

/// A mailbox message buffer.
///
/// The buffer must be 16-byte aligned.
#[repr(C, align(16))]
pub struct MailboxMessage {
    pub buffer_size: u32,
    pub request_response_code: u32,
    // Tags follow here
    // The last tag must be a 4-byte zero value (end tag)
}

/// Sends a mailbox message to the GPU and waits for a response.
///
/// The `message_ptr` must point to a 16-byte aligned `MailboxMessage` buffer.
pub fn send_mailbox_message(message_ptr: *mut MailboxMessage) -> Result<(), ()> {
    let ptr_addr = message_ptr as u32;

    // Check for 16-byte alignment
    if ptr_addr % 16 != 0 {
        return Err(()); // Not aligned
    }

    unsafe {
        // Wait until the mailbox is not full
        while ptr::read_volatile(MAILBOX_STATUS) & MAILBOX_FULL != 0 {}

        // Write the message address to the mailbox, along with the channel
        ptr::write_volatile(MAILBOX_WRITE, ptr_addr | MAILBOX_CHANNEL_PROPERTY_TAGS_ARM_TO_VC);

        // Wait for a response
        loop {
            // Wait until the mailbox is not empty
            while ptr::read_volatile(MAILBOX_STATUS) & MAILBOX_EMPTY != 0 {}

            // Read the response
            let response = ptr::read_volatile(MAILBOX_READ);

            // Check if the response is for our message and channel
            if (response & 0xF) == MAILBOX_CHANNEL_PROPERTY_TAGS_ARM_TO_VC {
                // Check if the GPU processed the message successfully
                if (*message_ptr).request_response_code == MBOX_RESPONSE_SUCCESS {
                    return Ok(());
                } else {
                    return Err(()); // GPU error
                }
            }
        }
    }
}
