"""Storage types for the voting node."""
import os

from asyncpg import Connection, Record
from loguru import logger
from pydantic import BaseModel

from .envvar import SECRET_SECRET
from .models.committee import Committee, ElectionKey
from .utils import decrypt_secret, encrypt_secret


class SecretDBStorage(BaseModel):
    """DB secret storage for secrets."""

    conn: Connection
    """Connection to DB storage."""

    class Config:
        """Pydantic model configuration parameters."""

        arbitrary_types_allowed = True

    async def get_committee(self, event_id: int) -> Committee:
        """Fetch the tally committee for the event_id.

        This method makes use of the `SECRET_SECRET` envvar.

        Raise exception if `SECRET_SECRET` is not defined.

        Raise exception if the tally committee is not found.
        """
        query = """
        SELECT
            tc.row_id as row_id,
            tc.event as event_id,
            ev.committee_size as size,
            ev.committee_threshold as threshold,
            tc.committee_pk as committee_pk,
            tc.committee_id as committee_id,
            member_crs as crs,
            tc.election_key as election_key
        FROM
            tally_committee tc
        INNER JOIN event ev
            ON tc.event = ev.row_id
        WHERE tc.event = $1"""

        try:
            record: Record | None = await self.conn.fetchrow(query, int(event_id))
            if record is None:
                raise Exception("expected to get tally committee in DB")
            logger.debug(f"fetched committee {record}")
        except:
            raise Exception("expected to get tally committee in DB")

        # fetch secret from envvar, fail if not present
        decrypt_pass = os.environ[SECRET_SECRET]

        # decrypt the CRS from the DB
        crs = decrypt_secret(record["crs"], decrypt_pass)
        committee = Committee(
            row_id=record["row_id"],
            event_id=int(record["event_id"]),
            size=record["size"],
            threshold=record["threshold"],
            crs=crs,
            committee_id=record["committee_id"],
            committee_pk=record["committee_pk"],
            election_key=ElectionKey(pubkey=record["election_key"]),
        )
        # fetch committee members
        query = """
        SELECT
            *
        FROM
            committee_member
        WHERE committee = $1"""
        rows = await self.conn.fetch(query, committee.row_id)
        if rows is None:
            raise Exception("expected committee members in DB")

        logger.debug(f"fetched committee members {rows}")

        def get_committee_member(row):
            pass

        # TODO: parse rows
        return committee

    async def save_committee(self, event_id: int, committee: Committee):
        """Save tally committee for the Committee.event.

        Tally committee information can be pushed to EventDB ONLY if there is no tally committee for  the given event.
        Return the `tally_committee.row_id` from the saved DB row.

        This method makes use of the `SECRET_SECRET` envvar.

        Raise exception if `SECRET_SECRET` is not defined.

        Raise exception if the tally committee already exists. There can only be one tally per event.
        """
        fields = "event, committee_pk, committee_id, member_crs, election_key"
        values = "$1, $2, $3, $4, $5"
        query = f"INSERT INTO tally_committee({fields}) VALUES({values}) RETURNING row_id"
        # fetch secret from envvar, fail if not present
        encrypt_pass = os.environ[SECRET_SECRET]
        # encrypt the Committee private key before adding to DB
        enc_pk = encrypt_secret(committee.committee_pk, encrypt_pass)
        # encrypt the CRS before adding to DB
        enc_crs = encrypt_secret(committee.crs, encrypt_pass)
        result = await self.conn.execute(
            query,
            event_id,
            committee.committee_id,
            enc_pk,
            enc_crs,
            committee.election_key.pubkey,
        )
        if result is None:
            raise Exception("failed to insert tally committee info to DB")
        logger.debug(f"tally committee info for event {event_id} added: ROW_ID {result}")

        # add committee members
        query = """INSERT INTO committee_member(
            committee,
            member_index,
            threshold,
            comm_pk,
            comm_sk,
            member_pk,
            member_sk
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7
        ) RETURNING row_id
        """
        _stmt = await self.conn.prepare(query)
