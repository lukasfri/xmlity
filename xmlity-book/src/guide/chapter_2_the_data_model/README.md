# Chapter 2 - The data model

The data model of Xmlity is heavily inspired by serde.

## Elements

The data-model of elements in Xmlity is built as a two-stage design, where the first stage of (de)serialization handles attributes and the second handles children. This is enforced by types so that both readers and writers can read in a logical direction, not having to jump between handling attributes and child elements.

## Attributes

## Groups
