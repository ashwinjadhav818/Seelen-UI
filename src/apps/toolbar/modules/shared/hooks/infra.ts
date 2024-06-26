import { ExtraCallbacksOnActivate, ExtraCallbacksOnLeave } from '../../../events';
import { useEffect, useRef } from 'react';

export function useAppBlur(cb: () => void, deps: any[] = []) {
  const key = useRef(crypto.randomUUID());
  useEffect(() => {
    ExtraCallbacksOnLeave.remove(key.current);
    ExtraCallbacksOnLeave.add(cb, key.current);
    return () => {
      ExtraCallbacksOnLeave.remove(key.current);
    };
  }, deps);
}

export function useAppActivation(cb: () => void, deps: any[] = []) {
  const key = useRef(crypto.randomUUID());
  useEffect(() => {
    ExtraCallbacksOnActivate.remove(key.current);
    ExtraCallbacksOnActivate.add(cb, key.current);
    return () => {
      ExtraCallbacksOnActivate.remove(key.current);
    };
  }, deps);
}