export interface Arrow {
  id: string;
  startX: number;
  startY: number;
  endX: number;
  endY: number;
  color: string;
  thickness: number;
  whiteStroke: boolean;
  dropShadow: boolean;
}

export interface ArrowSettings {
  color: string;
  thickness: number;
  whiteStroke: boolean;
  dropShadow: boolean;
}
