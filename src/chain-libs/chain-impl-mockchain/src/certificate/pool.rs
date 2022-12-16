use super::CertificateSlice;
use crate::key::{deserialize_public_key, deserialize_signature, GenesisPraosLeader};
use crate::rewards::TaxType;
use crate::transaction::{
    AccountIdentifier, Payload, PayloadAuthData, PayloadData, PayloadSlice,
    SingleAccountBindingSignature, TransactionBindingAuthData,
};
use chain_core::{
    packer::Codec,
    property::{Deserialize, DeserializeFromSlice, ReadError, Serialize, WriteError},
};
use chain_crypto::{digest::DigestOf, Blake2b256, Ed25519, PublicKey, Verification};
use chain_time::{DurationSeconds, TimeOffsetSeconds};
use std::marker::PhantomData;
use typed_bytes::{ByteArray, ByteBuilder};

/// Pool ID
pub type PoolId = PoolRegistrationHash;

/// Pool Registration Cryptographic Hash
pub type PoolRegistrationHash = DigestOf<Blake2b256, PoolRegistration>;

/// Hash of keys used for pool
pub type GenesisPraosLeaderHash = DigestOf<Blake2b256, GenesisPraosLeader>;

/// signatures with indices
pub type IndexSignatures = Vec<(u8, SingleAccountBindingSignature)>;

/// Pool information
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub struct PoolRegistration {
    /// A random value, for user purpose similar to a UUID.
    /// it may not be unique over a blockchain, so shouldn't be used a unique identifier
    pub serial: u128,
    /// Beginning of validity for this pool, this is used
    /// to keep track of the period of the expected key and the expiry
    pub start_validity: TimeOffsetSeconds,
    /// Permission system for this pool
    /// * Management threshold for owners, this need to be <= #owners and > 0.
    pub permissions: PoolPermissions,
    /// Owners of this pool
    pub owners: Vec<PublicKey<Ed25519>>,
    /// Operators of this pool
    pub operators: Box<[PublicKey<Ed25519>]>,
    /// Rewarding
    pub rewards: TaxType,
    /// Reward account
    pub reward_account: Option<AccountIdentifier>,
    /// Genesis Praos keys
    pub keys: GenesisPraosLeader,
}

/// Permission system related to the pool
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub struct PoolPermissions(u64);

pub type ManagementThreshold = u8;

const MANAGEMENT_THRESHOLD_BITMASK: u64 = 0b11_1111; // only support 32, reserved one for later extension if needed

#[allow(clippy::unusual_byte_groupings)]
const ALL_USED_BITMASK: u64 =
    0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00111111;

impl PoolPermissions {
    pub fn new(management_threshold: u8) -> PoolPermissions {
        let v = management_threshold as u64 & MANAGEMENT_THRESHOLD_BITMASK;
        PoolPermissions(v)
    }

    pub fn from_u64(v: u64) -> Option<PoolPermissions> {
        if (v & !ALL_USED_BITMASK) > 0 {
            None
        } else {
            Some(PoolPermissions(v))
        }
    }

    pub fn management_threshold(self) -> ManagementThreshold {
        (self.0 & MANAGEMENT_THRESHOLD_BITMASK) as ManagementThreshold
    }
}

/// Updating info for a pool
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolUpdate {
    pub pool_id: PoolId,
    pub last_pool_reg_hash: PoolRegistrationHash,
    pub new_pool_reg: PoolRegistration,
}

/// Retirement info for a pool
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolRetirement {
    pub pool_id: PoolId,
    pub retirement_time: TimeOffsetSeconds,
}

#[derive(Debug, Clone)]
pub enum PoolSignature {
    Operator(SingleAccountBindingSignature),
    Owners(PoolOwnersSignature),
}

/// Representant of a structure signed by a pool's owners
#[derive(Debug, Clone)]
pub struct PoolOwnersSignature {
    pub signatures: IndexSignatures,
}

pub type PoolOwnersSigned = PoolOwnersSignature;

impl PoolRegistration {
    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        let bb = bb
            .u128(self.serial)
            .u64(self.start_validity.into())
            .u64(self.permissions.0)
            .bytes(self.keys.vrf_public_key.as_ref())
            .bytes(self.keys.kes_public_key.as_ref())
            .iter8(&mut self.owners.iter(), |bb, o| bb.bytes(o.as_ref()))
            .iter8(&mut self.operators.iter(), |bb, o| bb.bytes(o.as_ref()))
            .sub(|sbb| self.rewards.serialize_in(sbb));

        match &self.reward_account {
            None => bb.u8(0),
            Some(AccountIdentifier::Single(pk)) => bb.u8(1).bytes(pk.as_ref().as_ref()),
            Some(AccountIdentifier::Multi(pk)) => bb.u8(2).bytes(pk.as_ref()),
        }
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }

    pub fn to_id(&self) -> PoolId {
        let ba = self.serialize();
        DigestOf::digest_byteslice(&ba.as_byteslice())
    }

    pub fn management_threshold(&self) -> u8 {
        self.permissions.management_threshold()
    }
}

impl PoolUpdate {
    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        bb.bytes(self.pool_id.as_ref())
            .bytes(self.last_pool_reg_hash.as_ref())
            .sub(|bb| self.new_pool_reg.serialize_in(bb))
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }
}

impl DeserializeFromSlice for PoolUpdate {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        let pool_id = <[u8; 32]>::deserialize(codec)?.into();
        let last_pool_reg_hash = <[u8; 32]>::deserialize(codec)?.into();
        let new_pool_reg = PoolRegistration::deserialize_from_slice(codec)?;
        Ok(PoolUpdate {
            pool_id,
            last_pool_reg_hash,
            new_pool_reg,
        })
    }
}

impl PoolRetirement {
    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        bb.bytes(self.pool_id.as_ref())
            .u64(self.retirement_time.into())
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }
}

impl Deserialize for PoolRetirement {
    fn deserialize<R: std::io::Read>(codec: &mut Codec<R>) -> Result<Self, ReadError> {
        let pool_id = <[u8; 32]>::deserialize(codec)?.into();
        let retirement_time = DurationSeconds::from(codec.get_be_u64()?).into();
        Ok(PoolRetirement {
            pool_id,
            retirement_time,
        })
    }
}

impl Serialize for PoolUpdate {
    fn serialized_size(&self) -> usize {
        self.serialize().as_slice().len()
    }

    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        codec.put_bytes(self.serialize().as_slice())
    }
}

impl Serialize for PoolRetirement {
    fn serialized_size(&self) -> usize {
        self.serialize().as_slice().len()
    }

    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        codec.put_bytes(self.serialize().as_slice())
    }
}

impl Payload for PoolUpdate {
    const HAS_DATA: bool = true;
    const HAS_AUTH: bool = true;
    type Auth = PoolSignature;
    fn payload_data(&self) -> PayloadData<Self> {
        PayloadData(
            self.serialize_in(ByteBuilder::new())
                .finalize_as_vec()
                .into(),
            PhantomData,
        )
    }
    fn payload_auth_data(auth: &Self::Auth) -> PayloadAuthData<Self> {
        PayloadAuthData(
            auth.serialize_in(ByteBuilder::new())
                .finalize_as_vec()
                .into(),
            PhantomData,
        )
    }
    fn payload_to_certificate_slice(p: PayloadSlice<'_, Self>) -> Option<CertificateSlice<'_>> {
        Some(CertificateSlice::from(p))
    }
}

impl Payload for PoolRetirement {
    const HAS_DATA: bool = true;
    const HAS_AUTH: bool = true;
    type Auth = PoolSignature;
    fn payload_data(&self) -> PayloadData<Self> {
        PayloadData(
            self.serialize_in(ByteBuilder::new())
                .finalize_as_vec()
                .into(),
            PhantomData,
        )
    }
    fn payload_auth_data(auth: &Self::Auth) -> PayloadAuthData<Self> {
        PayloadAuthData(
            auth.serialize_in(ByteBuilder::new())
                .finalize_as_vec()
                .into(),
            PhantomData,
        )
    }
    fn payload_to_certificate_slice(p: PayloadSlice<'_, Self>) -> Option<CertificateSlice<'_>> {
        Some(CertificateSlice::from(p))
    }
}

impl Serialize for PoolRegistration {
    fn serialized_size(&self) -> usize {
        self.serialize().as_slice().len()
    }

    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        codec.put_bytes(self.serialize().as_slice())
    }
}

impl DeserializeFromSlice for PoolRegistration {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        let serial = codec.get_be_u128()?;
        let start_validity = DurationSeconds::from(codec.get_be_u64()?).into();
        let permissions = PoolPermissions::from_u64(codec.get_be_u64()?).ok_or_else(|| {
            ReadError::StructureInvalid("permission value not correct".to_string())
        })?;
        let keys = GenesisPraosLeader::deserialize_from_slice(codec)?;

        let owners_nb = codec.get_u8()?;
        let mut owners = Vec::with_capacity(owners_nb as usize);
        for _ in 0..owners_nb {
            owners.push(deserialize_public_key(codec)?);
        }

        let operators_nb = codec.get_u8()?;
        let mut operators = Vec::with_capacity(operators_nb as usize);
        for _ in 0..operators_nb {
            operators.push(deserialize_public_key(codec)?);
        }

        let rewards = TaxType::read_frombuf(codec)?;
        let reward_account = match codec.get_u8()? {
            0 => None,
            1 => {
                let pk = deserialize_public_key(codec)?;
                Some(AccountIdentifier::Single(pk.into()))
            }
            2 => {
                let mut pk = [0u8; 32];
                codec.copy_to_slice(&mut pk)?;
                Some(AccountIdentifier::Multi(pk.into()))
            }
            n => return Err(ReadError::UnknownTag(n as u32)),
        };

        let info = Self {
            serial,
            start_validity,
            permissions,
            owners,
            operators: operators.into(),
            rewards,
            reward_account,
            keys,
        };
        Ok(info)
    }
}

impl Payload for PoolRegistration {
    const HAS_DATA: bool = true;
    const HAS_AUTH: bool = true;
    type Auth = PoolSignature;
    fn payload_data(&self) -> PayloadData<Self> {
        PayloadData(
            self.serialize_in(ByteBuilder::new())
                .finalize_as_vec()
                .into(),
            PhantomData,
        )
    }

    fn payload_auth_data(auth: &Self::Auth) -> PayloadAuthData<Self> {
        PayloadAuthData(
            auth.serialize_in(ByteBuilder::new())
                .finalize_as_vec()
                .into(),
            PhantomData,
        )
    }

    fn payload_to_certificate_slice(p: PayloadSlice<'_, Self>) -> Option<CertificateSlice<'_>> {
        Some(CertificateSlice::from(p))
    }
}

impl PoolSignature {
    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        match self {
            PoolSignature::Operator(op) => bb.u8(0).bytes(op.0.as_ref()),
            PoolSignature::Owners(owners) => {
                assert!(!owners.signatures.is_empty());
                assert!(owners.signatures.len() < 256);
                bb.u8(1).iter8(&mut owners.signatures.iter(), |bb, (i, s)| {
                    bb.u8(*i).bytes(s.as_ref())
                })
            }
        }
    }

    pub fn verify<'a>(
        &self,
        pool_info: &PoolRegistration,
        verify_data: &TransactionBindingAuthData<'a>,
    ) -> Verification {
        match self {
            PoolSignature::Operator(_) => Verification::Failed,
            PoolSignature::Owners(owners) => owners.verify(pool_info, verify_data),
        }
    }
}

impl PoolOwnersSignature {
    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        bb.iter8(&mut self.signatures.iter(), |bb, (i, s)| {
            bb.u8(*i).bytes(s.as_ref())
        })
    }

    pub fn verify<'a>(
        &self,
        pool_info: &PoolRegistration,
        verify_data: &TransactionBindingAuthData<'a>,
    ) -> Verification {
        // fast track if we don't meet the management threshold already
        if self.signatures.len() < pool_info.management_threshold() as usize {
            return Verification::Failed;
        }

        let mut present = vec![false; pool_info.owners.len()];
        let mut signatories = 0;

        for (i, sig) in self.signatures.iter() {
            let i = *i as usize;
            // Check for out of bounds indices
            if i >= pool_info.owners.len() {
                return Verification::Failed;
            }

            // If already present, then we have a duplicate hence fail
            if present[i] {
                return Verification::Failed;
            } else {
                present[i] = true;
            }

            // Verify the cryptographic signature of a signatory
            let pk = &pool_info.owners[i];
            if sig.verify_slice(pk, verify_data) == Verification::Failed {
                return Verification::Failed;
            }
            signatories += 1
        }

        // check if we seen enough unique signatures; it is a redundant check
        // from the duplicated check + the threshold check
        if signatories < pool_info.management_threshold() as usize {
            return Verification::Failed;
        }

        Verification::Success
    }
}

impl DeserializeFromSlice for PoolOwnersSigned {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        let sigs_nb = codec.get_u8()? as usize;
        if sigs_nb == 0 {
            return Err(ReadError::StructureInvalid(
                "pool owner signature with 0 signatures".to_string(),
            ));
        }
        let mut signatures = Vec::new();
        for _ in 0..sigs_nb {
            let nb = codec.get_u8()?;
            let sig = deserialize_signature(codec)?;
            signatures.push((nb, SingleAccountBindingSignature(sig)))
        }
        Ok(PoolOwnersSigned { signatures })
    }
}

impl DeserializeFromSlice for PoolSignature {
    fn deserialize_from_slice(codec: &mut Codec<&[u8]>) -> Result<Self, ReadError> {
        match codec.get_u8()? {
            0 => {
                let sig = deserialize_signature(codec)?;
                Ok(PoolSignature::Operator(SingleAccountBindingSignature(sig)))
            }
            1 => PoolOwnersSigned::deserialize_from_slice(codec).map(PoolSignature::Owners),
            code => Err(ReadError::UnknownTag(code as u32)),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{PoolOwnersSigned, PoolPermissions};
    use crate::{
        chaintypes::HeaderId,
        date::BlockDate,
        key::EitherEd25519SecretKey,
        testing::{
            builders::{make_witness, StakePoolBuilder},
            data::AddressData,
            TestGen,
        },
        transaction::{
            Input, NoExtra, SingleAccountBindingSignature, TransactionSignDataHash, TxBuilder,
            Witness,
        },
        value::Value,
    };
    use chain_addr::Discrimination;
    use chain_crypto::{Ed25519, PublicKey, Verification};
    use std::iter;

    #[derive(Clone, Debug)]
    pub struct PoolOwnersWithSignatures {
        pub owners: Vec<AddressData>,
        pub signatories: Vec<(u8, AddressData)>,
    }

    impl PoolOwnersWithSignatures {
        pub fn new(owners: Vec<AddressData>, signatories: Vec<(u8, AddressData)>) -> Self {
            PoolOwnersWithSignatures {
                owners,
                signatories,
            }
        }

        pub fn owners_pks(&self) -> Vec<PublicKey<Ed25519>> {
            self.owners
                .iter()
                .cloned()
                .map(|x| x.public_key())
                .collect()
        }

        pub fn indexed_signatories_sks(&self) -> Vec<(u8, EitherEd25519SecretKey)> {
            self.signatories
                .iter()
                .cloned()
                .map(|(i, x)| (i, x.private_key()))
                .collect()
        }

        pub fn witnesses(
            &self,
            block0_hash: HeaderId,
            transaction_hash: &TransactionSignDataHash,
        ) -> Vec<Witness> {
            self.signatories
                .iter()
                .map(|(_, x)| make_witness(&block0_hash, x, transaction_hash))
                .collect()
        }

        pub fn inputs(&self) -> Vec<Input> {
            self.signatories
                .iter()
                .map(|(_, x)| x.make_input(Value::zero(), None))
                .collect()
        }
    }

    /// Signatures count is grater than pool permissions and signatories have correct indexes
    #[test]
    pub fn pool_owners_correct_signature() {
        let owners_len = 8;
        let pool_permissions = PoolPermissions::new(4);
        let owners: Vec<AddressData> =
            iter::from_fn(|| Some(AddressData::account(Discrimination::Test)))
                .take(owners_len)
                .collect();

        let signatories: Vec<(u8, AddressData)> = vec![
            (0u8, owners[0].clone()),
            (1u8, owners[1].clone()),
            (2u8, owners[2].clone()),
            (3u8, owners[3].clone()),
        ];

        let pool_owner_with_sign = PoolOwnersWithSignatures::new(owners, signatories);
        test_verify(
            pool_owner_with_sign,
            pool_permissions,
            true,
            "verification should succeed because signatories are larger than pool permissions",
        );
    }

    /// duplicated owner, which cause that signatures are less than owners
    #[test]
    pub fn pool_owners_signature_duplicated_owner() {
        let owners_len = 8;
        let pool_permissions = PoolPermissions::new(4);
        let owners: Vec<AddressData> =
            iter::from_fn(|| Some(AddressData::account(Discrimination::Test)))
                .take(owners_len)
                .collect();

        let signatories: Vec<(u8, AddressData)> = vec![
            (0u8, owners[0].clone()),
            (0u8, owners[0].clone()),
            (2u8, owners[2].clone()),
            (3u8, owners[3].clone()),
        ];

        let pool_owner_with_sign = PoolOwnersWithSignatures::new(owners, signatories);
        test_verify(pool_owner_with_sign, pool_permissions, false, "verification should fail because one of signature is duplicated which cause that we have only
        3 valid signatures, while 4 are needed");
    }

    /// duplicated owner, but signatures are ok
    #[test]
    pub fn pool_owners_signature_extra_duplicated_signature() {
        let owners_len = 8;
        let pool_permissions = PoolPermissions::new(4);
        let owners: Vec<AddressData> =
            iter::from_fn(|| Some(AddressData::account(Discrimination::Test)))
                .take(owners_len)
                .collect();

        let signatories: Vec<(u8, AddressData)> = vec![
            (0u8, owners[0].clone()),
            (0u8, owners[0].clone()),
            (1u8, owners[1].clone()),
            (2u8, owners[2].clone()),
            (3u8, owners[3].clone()),
        ];

        let pool_owner_with_sign = PoolOwnersWithSignatures::new(owners, signatories);
        test_verify(pool_owner_with_sign, pool_permissions, false, "verification should fail there is duplicated signature even if unique signatures count is larger than
        permission threshold");
    }

    #[test]
    pub fn pool_owners_signature_wrong_signature() {
        let owners_len = 2;
        let pool_permissions = PoolPermissions::new(1);
        let owners: Vec<AddressData> =
            iter::from_fn(|| Some(AddressData::account(Discrimination::Test)))
                .take(owners_len)
                .collect();

        let signatories: Vec<(u8, AddressData)> = vec![
            (0u8, AddressData::account(Discrimination::Test)),
            (1u8, owners[1].clone()),
        ];

        let pool_owner_with_sign = PoolOwnersWithSignatures::new(owners, signatories);
        test_verify(
            pool_owner_with_sign,
            pool_permissions,
            false,
            "verification should fail due to invalid signature",
        );
    }

    #[test]
    pub fn pool_owners_signature_too_many_signatures() {
        let owners_len = 2;
        let pool_permissions = PoolPermissions::new(1);
        let owners: Vec<AddressData> =
            iter::from_fn(|| Some(AddressData::account(Discrimination::Test)))
                .take(owners_len)
                .collect();

        let signatories: Vec<(u8, AddressData)> = vec![
            (0u8, owners[0].clone()),
            (1u8, owners[1].clone()),
            (2u8, AddressData::account(Discrimination::Test)),
        ];

        let pool_owner_with_sign = PoolOwnersWithSignatures::new(owners, signatories);
        test_verify(
            pool_owner_with_sign,
            pool_permissions,
            false,
            "verification should fail due to extraordinary signature",
        );
    }

    #[test]
    pub fn pool_owners_signature_too_few_signatures() {
        let owners_len = 4;
        let pool_permissions = PoolPermissions::new(2);
        let owners: Vec<AddressData> =
            iter::from_fn(|| Some(AddressData::account(Discrimination::Test)))
                .take(owners_len)
                .collect();

        let signatories: Vec<(u8, AddressData)> = vec![(0u8, owners[0].clone())];

        let pool_owner_with_sign = PoolOwnersWithSignatures::new(owners, signatories);
        test_verify(
            pool_owner_with_sign,
            pool_permissions,
            false,
            "verification should fail due to non sufficient signatures",
        );
    }

    #[test]
    pub fn pool_owners_signature_different_order() {
        let owners_len = 8;
        let pool_permissions = PoolPermissions::new(4);
        let owners: Vec<AddressData> =
            iter::from_fn(|| Some(AddressData::account(Discrimination::Test)))
                .take(owners_len)
                .collect();

        let signatories: Vec<(u8, AddressData)> = vec![
            (0u8, owners[4].clone()),
            (1u8, owners[3].clone()),
            (2u8, owners[1].clone()),
            (3u8, owners[2].clone()),
        ];

        let pool_owner_with_sign = PoolOwnersWithSignatures::new(owners, signatories);
        test_verify(
            pool_owner_with_sign,
            pool_permissions,
            false,
            "verification should fail due to incorrect signatures indexes",
        );
    }

    #[test]
    pub fn pool_owners_signature_no_owners() {
        let owners_len = 0;
        let pool_permissions = PoolPermissions::new(0);
        let owners: Vec<AddressData> =
            iter::from_fn(|| Some(AddressData::account(Discrimination::Test)))
                .take(owners_len)
                .collect();

        let signatories: Vec<(u8, AddressData)> = vec![];

        let pool_owner_with_sign = PoolOwnersWithSignatures::new(owners, signatories);
        test_verify(
            pool_owner_with_sign,
            pool_permissions,
            true,
            "verification should fail due no owners",
        );
    }

    #[test]
    pub fn pool_owners_signature_zero_permissions() {
        let owners_len = 8;
        let pool_permissions = PoolPermissions::new(0);
        let owners: Vec<AddressData> =
            iter::from_fn(|| Some(AddressData::account(Discrimination::Test)))
                .take(owners_len)
                .collect();

        let signatories: Vec<(u8, AddressData)> = vec![(0u8, owners[0].clone())];

        let pool_owner_with_sign = PoolOwnersWithSignatures::new(owners, signatories);
        test_verify(
            pool_owner_with_sign,
            pool_permissions,
            true,
            "verification should fail due to 0 limit for permissions",
        );
    }

    /// For given pool_owner_with_sign (which contains pool registration owners and subset of signatures derived from them)
    /// and given pool_permissions limit it tests verify method for PoolOwnersSigned struct
    fn test_verify(
        pool_owner_with_sign: PoolOwnersWithSignatures,
        pool_permissions: PoolPermissions,
        should_pass: bool,
        info: &str,
    ) {
        let stake_pool = StakePoolBuilder::new()
            .with_owners(pool_owner_with_sign.owners_pks())
            .with_pool_permissions(pool_permissions)
            .build();

        let builder = TxBuilder::new()
            .set_payload(&NoExtra)
            .set_expiry_date(BlockDate::first().next_epoch())
            .set_ios(&pool_owner_with_sign.inputs(), &[]);
        let auth_data_hash = builder.get_auth_data_for_witness().hash();
        let builder = builder
            .set_witnesses(&pool_owner_with_sign.witnesses(TestGen::hash(), &auth_data_hash));

        let auth_data = builder.get_auth_data();
        let mut sigs = Vec::new();
        for (i, key) in pool_owner_with_sign.indexed_signatories_sks() {
            let sig = SingleAccountBindingSignature::new(&auth_data, |d| key.sign_slice(d.0));
            sigs.push((i, sig))
        }
        let pool_owner_signed = PoolOwnersSigned { signatures: sigs };
        let verification = if should_pass {
            Verification::Success
        } else {
            Verification::Failed
        };
        assert_eq!(
            pool_owner_signed.verify(&stake_pool.info(), &builder.get_auth_data()),
            verification,
            "{}",
            info
        );
    }
}
