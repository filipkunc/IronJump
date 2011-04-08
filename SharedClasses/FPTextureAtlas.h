//
//  FPTextureAtlas.h
//  IronJump
//
//  Created by Filip Kunc on 4/17/10.
//  For license see LICENSE.TXT
//

#import "FPTexture.h"
#import "FPGraphics.h"

@interface FPTextureAtlas : NSObject
{
	FPTexture *texture;
	int tileSize;
}

@property (readonly) FPTexture *texture;
@property (readonly) int tileSize;
@property (readonly) int horizontalTileCount;
@property (readonly) int verticalTileCount;

- (id)initWithFile:(NSString *)fileName convertToAlpha:(BOOL)convertToAlpha tileSize:(int)aTileSize;

@end
