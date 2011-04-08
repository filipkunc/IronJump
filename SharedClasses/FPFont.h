//
//  FPFont.h
//  IronJump
//
//  Created by Filip Kunc on 5/10/10.
//  For license see LICENSE.TXT
//

#import "FPTextureAtlas.h"

@interface FPFont : FPTextureAtlas
{
	float spacing;
}

@property (readwrite, assign) float spacing;

- (id)initWithFile:(NSString *)fileName tileSize:(int)aTileSize spacing:(float)aSpacing;
- (void)drawText:(NSString *)text atPoint:(CGPoint)pt;

@end
