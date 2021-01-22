// Copyright 2015-2016 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

//! Dh algrithem
//!
//! ```
//! use ring::dh::*;
//! use ring::rand;
//! let param = &DHPARAM_FFDHE2048;
//! let rng = rand::SystemRandom::new();
//! let my_private_key = DhContext::new(param, &rng);
//! let my_public_key = my_private_key.compute_public_key();
//!
//! let peer_public_key_buffer = [0u8; MAX_PUBLIC_KEY_LEN];
//! let peer_private_key = DhContext::new(param, &rng);
//! let peer_public_key = {
//!     peer_private_key.compute_public_key(&mut peer_public_key_buffer[..])
//! };
//!
//! let secret_key1 = my_private_key.compute_shared_key(&peer_public_key);
//! let secret_key2 = peer_private_key.compute_shared_key(&my_public_key);
//! ```

extern crate alloc;
use crate::rand;
use crate::arithmetic::bigint::*;
use crate::arithmetic::montgomery::{Unencoded,R};
use alloc::vec::Vec;


/// MAX_PUB_LICKEY_LEN for DH
pub const MAX_PUBLIC_KEY_LEN : usize = 1024usize;
/// MAX_SECRET_LEN for DH
pub const MAX_SECRET_LEN : usize = 64usize;

/// Indicates the dhparam
pub struct DhParam {
    /// prime
    pub p: &'static str,

    /// base
    pub g: &'static str,

    /// min secret length
    pub secret_len: usize,

    /// dhparam spec
    pub name: &'static str,
}

/// FFDHE2048
pub static DHPARAM_FFDHE2048: DhParam = DhParam {
    p: "FFFFFFFFFFFFFFFFADF85458A2BB4A9AAFDC5620273D3CF1\
D8B9C583CE2D3695A9E13641146433FBCC939DCE249B3EF9\
7D2FE363630C75D8F681B202AEC4617AD3DF1ED5D5FD6561\
2433F51F5F066ED0856365553DED1AF3B557135E7F57C935\
984F0C70E0E68B77E2A689DAF3EFE8721DF158A136ADE735\
30ACCA4F483A797ABC0AB182B324FB61D108A94BB2C8E3FB\
B96ADAB760D7F4681D4F42A3DE394DF4AE56EDE76372BB19\
0B07A7C8EE0A6D709E02FCE1CDF7E2ECC03404CD28342F61\
9172FE9CE98583FF8E4F1232EEF28183C3FE3B1B4C6FAD73\
3BB5FCBC2EC22005C58EF1837D1683B2C6F34A26C1B2EFFA\
886B423861285C97FFFFFFFFFFFFFFFF",
    g: "02",
    secret_len: 225 / 8 + 2,
    name: "ffdhe2048"
};

/// FFDE3072
pub static DHPARAM_FFDHE3072: DhParam = DhParam {
    p: "FFFFFFFFFFFFFFFFC90FDAA22168C234C4C6628B80DC1CD1\
    29024E088A67CC74020BBEA63B139B22514A08798E3404DD\
    EF9519B3CD3A431B302B0A6DF25F14374FE1356D6D51C245\
    E485B576625E7EC6F44C42E9A637ED6B0BFF5CB6F406B7ED\
    EE386BFB5A899FA5AE9F24117C4B1FE649286651ECE45B3D\
    C2007CB8A163BF0598DA48361C55D39A69163FA8FD24CF5F\
    83655D23DCA3AD961C62F356208552BB9ED529077096966D\
    670C354E4ABC9804F1746C08CA18217C32905E462E36CE3B\
    E39E772C180E86039B2783A2EC07A28FB5C55DF06F4C52C9\
    DE2BCBF6955817183995497CEA956AE515D2261898FA0510\
    15728E5A8AAAC42DAD33170D04507A33A85521ABDF1CBA64\
    ECFB850458DBEF0A8AEA71575D060C7DB3970F85A6E1E4C7\
    ABF5AE8CDB0933D71E8C94E04A25619DCEE3D2261AD2EE6B\
    F12FFA06D98A0864D87602733EC86A64521F2B18177B200C\
    BBE117577A615D6C770988C0BAD946E208E24FA074E5AB31\
    43DB5BFCE0FD108E4B82D120A93AD2CAFFFFFFFFFFFFFFFF",
    g: "02",
    secret_len: 275 / 8 + 2,
    name: "ffdhe3072"
};

/// FFDE4096
pub static DHPARAM_FFDHE4096: DhParam = DhParam {
    p: "FFFFFFFFFFFFFFFFADF85458A2BB4A9AAFDC5620273D3CF1\
    D8B9C583CE2D3695A9E13641146433FBCC939DCE249B3EF9\
    7D2FE363630C75D8F681B202AEC4617AD3DF1ED5D5FD6561\
    2433F51F5F066ED0856365553DED1AF3B557135E7F57C935\
    984F0C70E0E68B77E2A689DAF3EFE8721DF158A136ADE735\
    30ACCA4F483A797ABC0AB182B324FB61D108A94BB2C8E3FB\
    B96ADAB760D7F4681D4F42A3DE394DF4AE56EDE76372BB19\
    0B07A7C8EE0A6D709E02FCE1CDF7E2ECC03404CD28342F61\
    9172FE9CE98583FF8E4F1232EEF28183C3FE3B1B4C6FAD73\
    3BB5FCBC2EC22005C58EF1837D1683B2C6F34A26C1B2EFFA\
    886B4238611FCFDCDE355B3B6519035BBC34F4DEF99C0238\
    61B46FC9D6E6C9077AD91D2691F7F7EE598CB0FAC186D91C\
    AEFE130985139270B4130C93BC437944F4FD4452E2D74DD3\
    64F2E21E71F54BFF5CAE82AB9C9DF69EE86D2BC522363A0D\
    ABC521979B0DEADA1DBF9A42D5C4484E0ABCD06BFA53DDEF\
    3C1B20EE3FD59D7C25E41D2B669E1EF16E6F52C3164DF4FB\
    7930E9E4E58857B6AC7D5F42D69F6D187763CF1D55034004\
    87F55BA57E31CC7A7135C886EFB4318AED6A1E012D9E6832\
    A907600A918130C46DC778F971AD0038092999A333CB8B7A\
    1A1DB93D7140003C2A4ECEA9F98D0ACC0A8291CDCEC97DCF\
    8EC9B55A7F88A46B4DB5A851F44182E1C68A007E5E655F6A\
    FFFFFFFFFFFFFFFF",
    g: "02",
    secret_len: 325 / 8 + 2,
    name: "ffdhe4096"
};

/// FFDE6144
pub static DHPARAM_FFDHE6144: DhParam = DhParam {
    p: "FFFFFFFFFFFFFFFFADF85458A2BB4A9AAFDC5620273D3CF1\
    D8B9C583CE2D3695A9E13641146433FBCC939DCE249B3EF9\
    7D2FE363630C75D8F681B202AEC4617AD3DF1ED5D5FD6561\
    2433F51F5F066ED0856365553DED1AF3B557135E7F57C935\
    984F0C70E0E68B77E2A689DAF3EFE8721DF158A136ADE735\
    30ACCA4F483A797ABC0AB182B324FB61D108A94BB2C8E3FB\
    B96ADAB760D7F4681D4F42A3DE394DF4AE56EDE76372BB19\
    0B07A7C8EE0A6D709E02FCE1CDF7E2ECC03404CD28342F61\
    9172FE9CE98583FF8E4F1232EEF28183C3FE3B1B4C6FAD73\
    3BB5FCBC2EC22005C58EF1837D1683B2C6F34A26C1B2EFFA\
    886B4238611FCFDCDE355B3B6519035BBC34F4DEF99C0238\
    61B46FC9D6E6C9077AD91D2691F7F7EE598CB0FAC186D91C\
    AEFE130985139270B4130C93BC437944F4FD4452E2D74DD3\
    64F2E21E71F54BFF5CAE82AB9C9DF69EE86D2BC522363A0D\
    ABC521979B0DEADA1DBF9A42D5C4484E0ABCD06BFA53DDEF\
    3C1B20EE3FD59D7C25E41D2B669E1EF16E6F52C3164DF4FB\
    7930E9E4E58857B6AC7D5F42D69F6D187763CF1D55034004\
    87F55BA57E31CC7A7135C886EFB4318AED6A1E012D9E6832\
    A907600A918130C46DC778F971AD0038092999A333CB8B7A\
    1A1DB93D7140003C2A4ECEA9F98D0ACC0A8291CDCEC97DCF\
    8EC9B55A7F88A46B4DB5A851F44182E1C68A007E5E0DD902\
    0BFD64B645036C7A4E677D2C38532A3A23BA4442CAF53EA6\
    3BB454329B7624C8917BDD64B1C0FD4CB38E8C334C701C3A\
    CDAD0657FCCFEC719B1F5C3E4E46041F388147FB4CFDB477\
    A52471F7A9A96910B855322EDB6340D8A00EF092350511E3\
    0ABEC1FFF9E3A26E7FB29F8C183023C3587E38DA0077D9B4\
    763E4E4B94B2BBC194C6651E77CAF992EEAAC0232A281BF6\
    B3A739C1226116820AE8DB5847A67CBEF9C9091B462D538C\
    D72B03746AE77F5E62292C311562A846505DC82DB854338A\
    E49F5235C95B91178CCF2DD5CACEF403EC9D1810C6272B04\
    5B3B71F9DC6B80D63FDD4A8E9ADB1E6962A69526D43161C1\
    A41D570D7938DAD4A40E329CD0E40E65FFFFFFFFFFFFFFFF",
    g: "02",
    secret_len: 375 / 8 + 2,
    name: "ffdhe6144"
};

/// FFDE8192
pub static DHPARAM_FFDHE8192: DhParam = DhParam {
    p: "FFFFFFFFFFFFFFFFADF85458A2BB4A9AAFDC5620273D3CF1\
    D8B9C583CE2D3695A9E13641146433FBCC939DCE249B3EF9\
    7D2FE363630C75D8F681B202AEC4617AD3DF1ED5D5FD6561\
    2433F51F5F066ED0856365553DED1AF3B557135E7F57C935\
    984F0C70E0E68B77E2A689DAF3EFE8721DF158A136ADE735\
    30ACCA4F483A797ABC0AB182B324FB61D108A94BB2C8E3FB\
    B96ADAB760D7F4681D4F42A3DE394DF4AE56EDE76372BB19\
    0B07A7C8EE0A6D709E02FCE1CDF7E2ECC03404CD28342F61\
    9172FE9CE98583FF8E4F1232EEF28183C3FE3B1B4C6FAD73\
    3BB5FCBC2EC22005C58EF1837D1683B2C6F34A26C1B2EFFA\
    886B4238611FCFDCDE355B3B6519035BBC34F4DEF99C0238\
    61B46FC9D6E6C9077AD91D2691F7F7EE598CB0FAC186D91C\
    AEFE130985139270B4130C93BC437944F4FD4452E2D74DD3\
    64F2E21E71F54BFF5CAE82AB9C9DF69EE86D2BC522363A0D\
    ABC521979B0DEADA1DBF9A42D5C4484E0ABCD06BFA53DDEF\
    3C1B20EE3FD59D7C25E41D2B669E1EF16E6F52C3164DF4FB\
    7930E9E4E58857B6AC7D5F42D69F6D187763CF1D55034004\
    87F55BA57E31CC7A7135C886EFB4318AED6A1E012D9E6832\
    A907600A918130C46DC778F971AD0038092999A333CB8B7A\
    1A1DB93D7140003C2A4ECEA9F98D0ACC0A8291CDCEC97DCF\
    8EC9B55A7F88A46B4DB5A851F44182E1C68A007E5E0DD902\
    0BFD64B645036C7A4E677D2C38532A3A23BA4442CAF53EA6\
    3BB454329B7624C8917BDD64B1C0FD4CB38E8C334C701C3A\
    CDAD0657FCCFEC719B1F5C3E4E46041F388147FB4CFDB477\
    A52471F7A9A96910B855322EDB6340D8A00EF092350511E3\
    0ABEC1FFF9E3A26E7FB29F8C183023C3587E38DA0077D9B4\
    763E4E4B94B2BBC194C6651E77CAF992EEAAC0232A281BF6\
    B3A739C1226116820AE8DB5847A67CBEF9C9091B462D538C\
    D72B03746AE77F5E62292C311562A846505DC82DB854338A\
    E49F5235C95B91178CCF2DD5CACEF403EC9D1810C6272B04\
    5B3B71F9DC6B80D63FDD4A8E9ADB1E6962A69526D43161C1\
    A41D570D7938DAD4A40E329CCFF46AAA36AD004CF600C838\
    1E425A31D951AE64FDB23FCEC9509D43687FEB69EDD1CC5E\
    0B8CC3BDF64B10EF86B63142A3AB8829555B2F747C932665\
    CB2C0F1CC01BD70229388839D2AF05E454504AC78B758282\
    2846C0BA35C35F5C59160CC046FD8251541FC68C9C86B022\
    BB7099876A460E7451A8A93109703FEE1C217E6C3826E52C\
    51AA691E0E423CFC99E9E31650C1217B624816CDAD9A95F9\
    D5B8019488D9C0A0A1FE3075A577E23183F81D4A3F2FA457\
    1EFC8CE0BA8A4FE8B6855DFE72B0A66EDED2FBABFBE58A30\
    FAFABE1C5D71A87E2F741EF8C1FE86FEA6BBFDE530677F0D\
    97D11D49F7A8443D0822E506A9F4614E011E2A94838FF88C\
    D68C8BB7C5C6424CFFFFFFFFFFFFFFFF",
    g: "02",
    secret_len: 400 / 8 + 2,
    name: "ffdhe8192"
};

fn from_hex_digit(d: u8) -> Result<u8, &'static str> {
    if d >= b'0' && d <= b'9' {
        Ok(d - b'0')
    } else if d >= b'a' && d <= b'f' {
        Ok(d - b'a' + 10u8)
    } else if d >= b'A' && d <= b'F' {
        Ok(d - b'A' + 10u8)
    } else {
        // Err(format!("Invalid hex digit '{}'", d as char))
        Err("Invalid hex digit")
    }
}

/// Get bytes from hex string
pub fn from_hex(hex_str: &str) -> Result<Vec<u8>, &'static str> {
    if hex_str.len() % 2 != 0 {
        return Err("Hex string does not have an even number of digits");
    }

    let mut result = Vec::with_capacity(hex_str.len() / 2);
    for digits in hex_str.as_bytes().chunks(2) {
        let hi = from_hex_digit(digits[0])?;
        let lo = from_hex_digit(digits[1])?;
        result.push((hi * 0x10) | lo);
    }
    Ok(result)
}

/// Dhcontext
/// Dh
pub struct DhContext {
    a: PrivateExponent<()>,                // private key
    ap: Elem<()>,                          // public key
    p: Modulus<()>,                        // prime
}

fn into_encoded<T>(a: Elem<T, Unencoded>, m: &Modulus<T>) -> Elem<T, R> {
    elem_mul(m.oneRR().as_ref(), a, m)
}

impl DhContext {
    /// new a DhContext
    pub fn new(param: &'static DhParam, rng: &dyn rand::SecureRandom) -> Option<Self> {
        let mut a = [0u8; MAX_SECRET_LEN];

        let a = &mut a[0..param.secret_len];
        rng.fill(a).ok()?;

        a[param.secret_len-1] |= 1;

        let p = {
            let inputhex = &from_hex(param.p).ok()?;
            let input = untrusted::Input::from(inputhex);
            let v = Modulus::<()>::from_be_bytes_with_bit_length(
            input).ok()?;
            v.0
        };

        let a = {
            PrivateExponent::<()>::from_be_bytes_padded(untrusted::Input::from(a), &p).ok()?
        };

        let g: Elem<(),Unencoded> =  Elem::<()>::from_be_bytes_padded(
            untrusted::Input::from(
                &from_hex(param.g).unwrap()
        ),&p).ok()?;

        let g = into_encoded(g, &p);
        let ap = elem_exp_consttime(g, &a, &p).ok()?;
        Some(DhContext {a, ap, p})
    }

    /// get public key bytes
    pub fn compute_public_key<'a>(&self, buffer: &'a mut [u8]) -> &'a [u8] {
        let pubkey = self.ap.to_bytes_be();
        let pubkey_slice = pubkey.as_slice();
        let len = pubkey.len();
        let res = &mut buffer[0..len];
        res.copy_from_slice(pubkey_slice);
        res
    }

    /// get share key bytes
    pub fn compute_shared_key(&self, _peer_public_key: &[u8]) -> Vec<u8> {
        // let peer_public_key = BigUint::from_bytes_le(peer_public_key);
        // peer_public_key.modpow(&self.a, &self.p).to_bytes_le()
        let p = {&self.p};
        let a = {&self.a};
        let peer_public_key: Elem<(),Unencoded> = Elem::<()>::from_be_bytes_padded(
            untrusted::Input::from(_peer_public_key), p).unwrap();

        let peer_public_key = into_encoded(peer_public_key, &p);
        let r = elem_exp_consttime(peer_public_key, a, p).unwrap();
        r.to_bytes_be()
    }
}

/// Performs a key agreement with an ephemeral private key and the given public
/// key.
pub fn agree_ephemeral<F, R>(
    my_private_key: DhContext,
    peer_public_key: &[u8],
    kdf: F,
) -> Option<R>
where
    F: FnOnce(&[u8]) -> Option<R>
{
    let secret_key = my_private_key.compute_shared_key(peer_public_key);
    kdf(secret_key.as_slice())
}

#[cfg(test)]
mod tests {
    use crate::dh::*;
    use crate::rand;
    #[test]
    fn test_dh1() {
        let param = &DHPARAM_FFDHE2048;
        let rng = rand::SystemRandom::new();
        let my_private_key = DhContext::new(param, &rng).unwrap();
        let mut my_public_key_buffer = [0; MAX_PUBLIC_KEY_LEN];
        let my_public_key = my_private_key.compute_public_key(&mut my_public_key_buffer[..]);

        let peer_private_key = DhContext::new(param, &rng).unwrap();
        let mut peer_public_key_buffer = [0; MAX_PUBLIC_KEY_LEN];
        let peer_public_key = {
            peer_private_key.compute_public_key(&mut peer_public_key_buffer[..])
        };

        let secret_key1 = my_private_key.compute_shared_key(&peer_public_key);
        let secret_key2 = peer_private_key.compute_shared_key(&my_public_key);
        assert_eq!(secret_key1, secret_key2);
    }

    #[test]
    fn test_dh2() {
        let param = &DHPARAM_FFDHE3072;
        let rng = rand::SystemRandom::new();
        let my_private_key = DhContext::new(param, &rng).unwrap();

        let mut peer_public_key_buffer = [0u8; MAX_PUBLIC_KEY_LEN];
        let peer_public_key = {
            let peer_private_key = DhContext::new(param, &rng).unwrap();
            peer_private_key.compute_public_key(&mut peer_public_key_buffer[..])
        };

        let _ = agree_ephemeral(my_private_key, peer_public_key,|_shared_key| {

            Some(())
        });
    }
}
