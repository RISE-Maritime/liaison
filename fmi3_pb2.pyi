from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class Empty(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class Instance(_message.Message):
    __slots__ = ("key",)
    KEY_FIELD_NUMBER: _ClassVar[int]
    key: int
    def __init__(self, key: _Optional[int] = ...) -> None: ...

class getValue64Request(_message.Message):
    __slots__ = ("key", "valueReferences")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUEREFERENCES_FIELD_NUMBER: _ClassVar[int]
    key: int
    valueReferences: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, key: _Optional[int] = ..., valueReferences: _Optional[_Iterable[int]] = ...) -> None: ...

class getFloat64Reply(_message.Message):
    __slots__ = ("values",)
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, values: _Optional[_Iterable[float]] = ...) -> None: ...

class enterInitializationModeRequest(_message.Message):
    __slots__ = ("key", "toleranceDefined", "tolerance", "startTime", "stopTimeDefined", "stopTime")
    KEY_FIELD_NUMBER: _ClassVar[int]
    TOLERANCEDEFINED_FIELD_NUMBER: _ClassVar[int]
    TOLERANCE_FIELD_NUMBER: _ClassVar[int]
    STARTTIME_FIELD_NUMBER: _ClassVar[int]
    STOPTIMEDEFINED_FIELD_NUMBER: _ClassVar[int]
    STOPTIME_FIELD_NUMBER: _ClassVar[int]
    key: int
    toleranceDefined: bool
    tolerance: float
    startTime: float
    stopTimeDefined: bool
    stopTime: float
    def __init__(self, key: _Optional[int] = ..., toleranceDefined: bool = ..., tolerance: _Optional[float] = ..., startTime: _Optional[float] = ..., stopTimeDefined: bool = ..., stopTime: _Optional[float] = ...) -> None: ...

class doStepRequest(_message.Message):
    __slots__ = ("key", "currentCommunicationPoint", "communicationStepSize")
    KEY_FIELD_NUMBER: _ClassVar[int]
    CURRENTCOMMUNICATIONPOINT_FIELD_NUMBER: _ClassVar[int]
    COMMUNICATIONSTEPSIZE_FIELD_NUMBER: _ClassVar[int]
    key: int
    currentCommunicationPoint: float
    communicationStepSize: float
    def __init__(self, key: _Optional[int] = ..., currentCommunicationPoint: _Optional[float] = ..., communicationStepSize: _Optional[float] = ...) -> None: ...

class Array(_message.Message):
    __slots__ = ("values",)
    VALUES_FIELD_NUMBER: _ClassVar[int]
    values: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, values: _Optional[_Iterable[float]] = ...) -> None: ...
